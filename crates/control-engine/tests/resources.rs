use controlab_engine::resources;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};
fn temp() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "controlab-resources-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&p).unwrap();
    p
}
#[test]
fn source_assets_and_confinement_work() {
    let expected = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets")
        .canonicalize()
        .unwrap();
    assert_eq!(resources::assets_root().unwrap(), expected);
    assert!(resources::asset_path(&["manifest.json"]).unwrap().is_file());
    assert!(resources::asset_path(&["..", "Cargo.toml"]).is_err());
    assert!(resources::asset_path(&["/tmp"]).is_err());
    // Missing files are valid resource paths, as in pathlib.resolve(strict=False).
    assert_eq!(
        resources::asset_path(&["not-yet-present", "resource.txt"]).unwrap(),
        expected.join("not-yet-present/resource.txt")
    );
}
#[test]
fn override_is_fail_closed_and_canonical_symlink_escape_is_rejected() {
    let p = temp();
    fs::write(p.join("manifest.json"), "{}").unwrap();
    let exe = std::env::current_exe().unwrap();
    let run = |root: &std::path::Path, mode: &str| {
        Command::new(&exe)
            .args(["--exact", "override_child", "--nocapture"])
            .env("CONTROLAB_ASSETS", root)
            .env("CONTROLAB_RESOURCE_TEST_MODE", mode)
            .output()
            .unwrap()
    };
    let valid = run(&p, "valid");
    assert!(
        valid.status.success(),
        "{}",
        String::from_utf8_lossy(&valid.stdout)
    );
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(std::env::temp_dir(), p.join("outside")).unwrap();
        let out = run(&p, "symlink");
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stdout)
        );
    }
    fs::remove_file(p.join("manifest.json")).unwrap();
    let invalid = run(&p, "invalid");
    assert!(
        invalid.status.success(),
        "{}",
        String::from_utf8_lossy(&invalid.stdout)
    );
    fs::remove_dir_all(p).unwrap();
}
#[test]
fn override_child() {
    let Ok(mode) = std::env::var("CONTROLAB_RESOURCE_TEST_MODE") else {
        return;
    };
    match mode.as_str() {
        "valid" => assert_eq!(
            resources::assets_root().unwrap(),
            PathBuf::from(std::env::var_os("CONTROLAB_ASSETS").unwrap())
                .canonicalize()
                .unwrap()
        ),
        "invalid" => assert!(resources::assets_root().is_err()),
        "symlink" => assert!(resources::asset_path(&["outside", "escaped-file"]).is_err()),
        _ => panic!("unknown mode"),
    }
}

#[test]
fn relocated_bundle_discovers_sibling_resources_without_source_tree() {
    let root = temp();
    let engine = root.join("engine");
    let assets = root.join("resources/assets");
    fs::create_dir_all(&engine).unwrap();
    fs::create_dir_all(&assets).unwrap();
    fs::write(assets.join("manifest.json"), "{}").unwrap();
    let copy = engine.join(std::env::current_exe().unwrap().file_name().unwrap());
    fs::copy(std::env::current_exe().unwrap(), &copy).unwrap();
    let run = |mode: &str| {
        Command::new(&copy)
            .args(["--exact", "bundle_child", "--nocapture"])
            .env_remove("CONTROLAB_ASSETS")
            .env("CONTROLAB_RESOURCE_BUNDLE_MODE", mode)
            .output()
            .unwrap()
    };
    let result = run("valid");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
    fs::remove_file(assets.join("manifest.json")).unwrap();
    let result = run("incomplete");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn bundle_child() {
    let Ok(mode) = std::env::var("CONTROLAB_RESOURCE_BUNDLE_MODE") else {
        return;
    };
    if mode == "incomplete" {
        assert!(resources::assets_root().is_err());
        return;
    }
    let exe = std::env::current_exe().unwrap();
    assert_eq!(
        resources::assets_root().unwrap(),
        exe.parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("resources/assets")
            .canonicalize()
            .unwrap()
    );
}
