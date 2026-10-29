#[allow(dead_code)]
#[path = "../src/tools.rs"]
mod tools;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "controlab-tools-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn put(root: &Path, name: &str, text: &str) {
    let p = root.join(name);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}
fn run(root: &Path, args: &[&str]) -> Result<(), String> {
    tools::run_at(
        &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        root,
    )
}
#[test]
fn version_updates_all_local_crates_and_preserves_registry_packages() {
    let t = Temp::new();
    put(
        &t.0,
        "Cargo.toml",
        "[workspace]\nmembers=[\"native\",\"xtask\",\"crates/engine\"]\n[workspace.package]\nversion=\"1.0.0\"\n",
    );
    for (dir, name) in [
        ("native", "controlab-native"),
        ("xtask", "xtask"),
        ("crates/engine", "controlab-engine"),
    ] {
        put(
            &t.0,
            &format!("{dir}/Cargo.toml"),
            &format!("[package]\nname=\"{name}\"\nversion.workspace=true\n"),
        );
    }
    put(
        &t.0,
        "Cargo.lock",
        "version=4\n[[package]]\nname=\"controlab-native\"\nversion=\"1.0.0\"\n[[package]]\nname=\"xtask\"\nversion=\"1.0.0\"\n[[package]]\nname=\"controlab-engine\"\nversion=\"1.0.0\"\n[[package]]\nname=\"other\"\nversion=\"1.0.0\"\nsource=\"registry+https://example.test\"\n",
    );
    run(&t.0, &["version", "v2.3.4"]).unwrap();
    let lock = fs::read_to_string(t.0.join("Cargo.lock")).unwrap();
    assert_eq!(
        lock.parse::<toml_edit::DocumentMut>().unwrap()["package"]
            .as_array_of_tables()
            .unwrap()
            .iter()
            .filter(|p| p["version"].as_str() == Some("2.3.4"))
            .count(),
        3
    );
    assert!(lock.contains("name=\"other\"\nversion=\"1.0.0\""));
    assert!(
        fs::read_to_string(t.0.join("Cargo.toml"))
            .unwrap()
            .contains("2.3.4")
    );
    let before = fs::read(t.0.join("Cargo.toml")).unwrap();
    assert!(run(&t.0, &["version", "2.3.4-beta"]).is_err());
    assert_eq!(fs::read(t.0.join("Cargo.toml")).unwrap(), before);
}
#[test]
fn source_guard_skips_generated_trees_and_rejects_frontend_dependencies() {
    let t = Temp::new();
    put(&t.0, "target/ignored.js", "");
    put(&t.0, "src/lib.rs", "");
    run(&t.0, &["check-sources"]).unwrap();
    put(&t.0, "src/accidental.TSX", "");
    assert!(
        run(&t.0, &["check-sources"])
            .unwrap_err()
            .contains("src/accidental.TSX")
    );
}
#[test]
fn package_never_overwrites_source_or_nonempty_output() {
    let t = Temp::new();
    put(&t.0, "dist/existing", "preserve");
    let output = t.0.join("dist").to_string_lossy().to_string();
    let error = run(&t.0, &["package", "--skip-build", "--output", &output]).unwrap_err();
    assert!(error.contains("vazia"), "{error}");
    assert_eq!(
        fs::read_to_string(t.0.join("dist/existing")).unwrap(),
        "preserve"
    );
    let error = run(
        &t.0,
        &["package", "--skip-build", "--output", t.0.to_str().unwrap()],
    )
    .unwrap_err();
    assert!(error.contains("origem"), "{error}");
}
#[test]
fn installer_dry_run_checks_layout_without_requiring_packaging_tools() {
    let t = Temp::new();
    put(
        &t.0,
        "Cargo.toml",
        "[workspace.package]\nversion=\"6.0.0\"\n",
    );
    put(
        &t.0,
        "bundle/bundle.json",
        "{\"ui\":\"controlab\",\"engine\":\"engine/controlab-engine\",\"resources\":\"resources/assets\"}",
    );
    put(&t.0, "bundle/controlab", "ui");
    put(&t.0, "bundle/engine/controlab-engine", "engine");
    put(
        &t.0,
        "packaging/linux/controlab.desktop",
        "[Desktop Entry]\nName=ControLAB\n",
    );
    put(&t.0, "packaging/icons/linux/256x256.png", "png");
    let args = [
        "installers",
        "deb",
        "rpm",
        "appimage",
        "--dry-run",
        "--bundle",
        t.0.join("bundle").to_str().unwrap(),
        "--output",
        t.0.join("release").to_str().unwrap(),
    ]
    .map(str::to_string);
    if cfg!(target_os = "linux") {
        tools::run_at(&args, &t.0).unwrap();
        assert_eq!(fs::read_dir(t.0.join("release")).unwrap().count(), 0);
    }
}

#[test]
fn linux_installer_layout_preserves_engine_resources_and_excludes_validation_capture() {
    if !cfg!(unix) {
        return;
    }
    let t = Temp::new();
    put(&t.0, "bundle/controlab", "ui");
    put(&t.0, "bundle/engine/controlab-engine", "engine");
    put(&t.0, "bundle/resources/assets/manifest.json", "{}");
    put(&t.0, "bundle/validation-ui.png", "private validation image");
    put(&t.0, "packaging/linux/controlab.desktop", "[Desktop Entry]");
    put(&t.0, "packaging/icons/linux/256x256.png", "icon");
    let stage = t.0.join("stage");
    tools::stage_linux(&t.0, &t.0.join("bundle"), &stage).unwrap();
    assert_eq!(
        fs::read_link(stage.join("usr/bin/controlab")).unwrap(),
        PathBuf::from("/opt/controlab/controlab")
    );
    assert_eq!(
        fs::read_to_string(stage.join("opt/controlab/engine/controlab-engine")).unwrap(),
        "engine"
    );
    assert!(
        stage
            .join("opt/controlab/resources/assets/manifest.json")
            .is_file()
    );
    assert!(
        stage
            .join("usr/share/applications/controlab.desktop")
            .is_file()
    );
    assert!(
        stage
            .join("usr/share/icons/hicolor/256x256/apps/controlab.png")
            .is_file()
    );
    assert!(!stage.join("opt/controlab/validation-ui.png").exists());
}
#[test]
fn package_copies_built_rust_engine_and_resources_without_python_freezer() {
    // Do not mutate process-wide build variables; this fixture is independent of any custom target tree.
    if std::env::var_os("CARGO_TARGET_DIR").is_some() {
        return;
    }
    let t = Temp::new();
    put(
        &t.0,
        "Cargo.toml",
        "[workspace.package]\nversion=\"6.0.0\"\n",
    );
    put(&t.0, "LICENSE", "MIT");
    put(&t.0, "assets/manifest.json", "{\"files\":{}}");
    put(&t.0, "assets/fonts/font.ttf", "font");
    let suffix = if cfg!(windows) { ".exe" } else { "" };
    put(
        &t.0,
        &format!("target/debug/controlab{suffix}"),
        "native UI",
    );
    put(
        &t.0,
        &format!("target/debug/controlab-engine{suffix}"),
        "Rust engine",
    );
    let output = t.0.join("bundle");
    run(
        &t.0,
        &[
            "package",
            "--skip-build",
            "--output",
            output.to_str().unwrap(),
        ],
    )
    .unwrap();
    let metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(output.join("bundle.json")).unwrap()).unwrap();
    assert_eq!(metadata["resources"], "resources/assets");
    assert_eq!(
        metadata["engine"],
        format!("engine/controlab-engine{suffix}")
    );
    assert_eq!(
        fs::read_to_string(output.join(metadata["engine"].as_str().unwrap())).unwrap(),
        "Rust engine"
    );
    assert_eq!(
        fs::read_to_string(output.join("resources/assets/fonts/font.ttf")).unwrap(),
        "font"
    );
    assert!(!output.join("engine/_internal").exists());
}
#[test]
fn version_preflight_does_not_modify_manifest_when_local_lock_entry_is_missing() {
    let t = Temp::new();
    let manifest = "[workspace]\nmembers=[\"xtask\"]\n[workspace.package]\nversion=\"1.0.0\"\n";
    put(&t.0, "Cargo.toml", manifest);
    put(
        &t.0,
        "xtask/Cargo.toml",
        "[package]\nname=\"xtask\"\nversion.workspace=true\n",
    );
    put(
        &t.0,
        "Cargo.lock",
        "version=4\n[[package]]\nname=\"unrelated\"\nversion=\"1.0.0\"\n",
    );
    assert!(
        run(&t.0, &["version", "2.0.0"])
            .unwrap_err()
            .contains("ausentes")
    );
    assert_eq!(
        fs::read_to_string(t.0.join("Cargo.toml")).unwrap(),
        manifest
    );
}
#[cfg(unix)]
#[test]
fn packaging_rejects_symlink_to_source() {
    use std::os::unix::fs::symlink;
    let t = Temp::new();
    let elsewhere = Temp::new();
    symlink(&t.0, elsewhere.0.join("disguised")).unwrap();
    let e = run(
        &t.0,
        &[
            "package",
            "--skip-build",
            "--output",
            elsewhere.0.join("disguised").to_str().unwrap(),
        ],
    )
    .unwrap_err();
    assert!(e.contains("origem"), "{e}");
}

#[cfg(unix)]
fn validation_fixture(root: &Path) {
    use std::os::unix::fs::PermissionsExt;
    put(
        root,
        "bundle.json",
        "{\"ui\":\"controlab\",\"engine\":\"engine/controlab-engine\",\"resources\":\"resources/assets\"}",
    );
    put(root, "resources/assets/manifest.json", "{\"files\":{}}");
    put(
        root,
        "engine/controlab-engine",
        r##"#!/bin/sh
[ -z "$PYTHONPATH$CONTROLAB_ASSETS$CONTROLAB_ENGINE" ] || exit 10
[ "${CONTROLAB_PYTHON##*/}" = no-python ] || exit 11
[ "${CONTROLAB_NATIVE_BRIDGE##*/}" = no-source-bridge ] || exit 12
case "$PWD" in *controlab-bundle-check-*) ;; *) exit 13;; esac
while IFS= read -r request; do
 case "$request" in
 *time_response*) echo '{"success":true,"title":"Time response"}' ;;
 *render_formulas*) echo '{"success":true,"images":["data:image/svg+xml;base64,PHN2Zz4="]}' ;;
 *) echo '{"success":true}' ;;
 esac
done
"##,
    );
    fs::set_permissions(
        root.join("engine/controlab-engine"),
        fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    put(root, "controlab", "#!/bin/sh\nexit 0\n");
    fs::set_permissions(root.join("controlab"), fs::Permissions::from_mode(0o755)).unwrap();
}
#[cfg(unix)]
#[test]
fn bundle_validation_runs_protocol_outside_source_and_rejects_engine_failures() {
    let t = Temp::new();
    validation_fixture(&t.0);
    run(&t.0, &["validate-bundle", t.0.to_str().unwrap()]).unwrap();
    put(&t.0, "engine/controlab-engine", "#!/bin/sh\nexit 7\n");
    let error = run(&t.0, &["validate-bundle", t.0.to_str().unwrap()]).unwrap_err();
    assert!(error.contains("falhou"), "{error}");
}
#[cfg(unix)]
#[test]
fn bundle_validation_rejects_digest_changes_and_stale_ui_screenshot() {
    let t = Temp::new();
    validation_fixture(&t.0);
    put(
        &t.0,
        "resources/assets/manifest.json",
        "{\"files\":{\"font.ttf\":\"bad-digest\"}}",
    );
    put(&t.0, "resources/assets/font.ttf", "font");
    assert!(
        run(&t.0, &["validate-bundle", t.0.to_str().unwrap()])
            .unwrap_err()
            .contains("SHA256")
    );
    put(&t.0, "resources/assets/manifest.json", "{\"files\":{}}");
    put(&t.0, "validation-ui.png", "stale screenshot");
    assert!(run(&t.0, &["validate-bundle", t.0.to_str().unwrap(), "--ui"]).is_err());
    assert!(!t.0.join("validation-ui.png").exists());
}

#[cfg(unix)]
#[test]
fn source_guard_rejects_symlinked_node_modules() {
    use std::os::unix::fs::symlink;
    let t = Temp::new();
    let dependencies = Temp::new();
    symlink(&dependencies.0, t.0.join("node_modules")).unwrap();
    assert!(
        run(&t.0, &["check-sources"])
            .unwrap_err()
            .contains("node_modules")
    );
}
#[test]
fn command_help_is_available_without_build_tools() {
    let t = Temp::new();
    for command in [
        "package",
        "installers",
        "version",
        "validate-bundle",
        "check-sources",
        "android",
    ] {
        run(&t.0, &[command, "--help"]).unwrap();
    }
}
#[test]
fn source_guard_accepts_argparse_equals_option_syntax() {
    let t = Temp::new();
    run(
        &t.0,
        &["check-sources", &format!("--root={}", t.0.display())],
    )
    .unwrap();
}

#[test]
fn android_incremental_output_replaces_only_requested_libraries() {
    let t = Temp::new();
    let output = t.0.join("native/android/app/build/generated/native/debug");
    put(
        &output,
        "arm64-v8a/libcontrolab_native.so",
        "old compiled lib",
    );
    put(&output, "arm64-v8a/keep.txt", "unrelated");
    put(&output, "x86_64/libcontrolab_native.so", "different ABI");
    put(&t.0, "build/new-library.so", "first build");
    for version in ["first build", "second build"] {
        let checked = tools::android_output(&t.0, &output).unwrap();
        put(&t.0, "build/new-library.so", version);
        tools::copy_android_library(
            &checked,
            "arm64-v8a",
            &t.0.join("build/new-library.so"),
            "libcontrolab_native.so",
        )
        .unwrap();
        assert_eq!(
            fs::read_to_string(output.join("arm64-v8a/libcontrolab_native.so")).unwrap(),
            version
        );
    }
    assert_eq!(
        fs::read_to_string(output.join("arm64-v8a/keep.txt")).unwrap(),
        "unrelated"
    );
    assert_eq!(
        fs::read_to_string(output.join("x86_64/libcontrolab_native.so")).unwrap(),
        "different ABI"
    );
    assert!(
        tools::copy_android_library(
            &output,
            "arm64-v8a",
            &t.0.join("build/new-library.so"),
            "keep.txt"
        )
        .is_err()
    );
    assert!(
        tools::copy_android_library(
            &output,
            "../escape",
            &t.0.join("build/new-library.so"),
            "libcontrolab_native.so"
        )
        .is_err()
    );
}
#[test]
fn android_output_rejects_repository_and_its_ancestors() {
    let t = Temp::new();
    assert!(tools::android_output(&t.0, &t.0).is_err());
    assert!(tools::android_output(&t.0, t.0.parent().unwrap()).is_err());
}
#[cfg(unix)]
#[test]
fn android_incremental_copy_rejects_symlink_escape_and_preserves_source() {
    use std::os::unix::fs::symlink;
    let t = Temp::new();
    let elsewhere = Temp::new();
    let output = t.0.join("generated/native");
    fs::create_dir_all(&output).unwrap();
    put(&t.0, "build/new-library.so", "new binary");
    put(&elsewhere.0, "libcontrolab_native.so", "preserve source");
    symlink(&elsewhere.0, output.join("arm64-v8a")).unwrap();
    assert!(
        tools::copy_android_library(
            &output,
            "arm64-v8a",
            &t.0.join("build/new-library.so"),
            "libcontrolab_native.so"
        )
        .is_err()
    );
    assert_eq!(
        fs::read_to_string(elsewhere.0.join("libcontrolab_native.so")).unwrap(),
        "preserve source"
    );
    fs::create_dir_all(output.join("x86_64")).unwrap();
    symlink(
        elsewhere.0.join("libcontrolab_native.so"),
        output.join("x86_64/libcontrolab_native.so"),
    )
    .unwrap();
    assert!(
        tools::copy_android_library(
            &output,
            "x86_64",
            &t.0.join("build/new-library.so"),
            "libcontrolab_native.so"
        )
        .is_err()
    );
    assert_eq!(
        fs::read_to_string(elsewhere.0.join("libcontrolab_native.so")).unwrap(),
        "preserve source"
    );
    symlink(&t.0, elsewhere.0.join("disguised-root")).unwrap();
    assert!(tools::android_output(&t.0, &elsewhere.0.join("disguised-root")).is_err());
}

#[test]
fn failed_android_copy_preserves_previous_library_and_cleans_its_temporary_file() {
    let t = Temp::new();
    let output = t.0.join("explicit-output");
    put(
        &output,
        "arm64-v8a/libcontrolab_native.so",
        "last successful build",
    );
    assert!(
        tools::copy_android_library(
            &output,
            "arm64-v8a",
            &t.0.join("missing.so"),
            "libcontrolab_native.so"
        )
        .is_err()
    );
    assert_eq!(
        fs::read_to_string(output.join("arm64-v8a/libcontrolab_native.so")).unwrap(),
        "last successful build"
    );
    assert_eq!(fs::read_dir(output.join("arm64-v8a")).unwrap().count(), 1);
}
#[cfg(unix)]
#[test]
fn android_library_replacement_does_not_truncate_a_hardlinked_source() {
    let t = Temp::new();
    let output = t.0.join("output");
    put(&t.0, "source/keep.so", "preserve unrelated source");
    put(&t.0, "build/new.so", "new compiled library");
    fs::create_dir_all(output.join("arm64-v8a")).unwrap();
    fs::hard_link(
        t.0.join("source/keep.so"),
        output.join("arm64-v8a/libcontrolab_native.so"),
    )
    .unwrap();
    tools::copy_android_library(
        &output,
        "arm64-v8a",
        &t.0.join("build/new.so"),
        "libcontrolab_native.so",
    )
    .unwrap();
    assert_eq!(
        fs::read_to_string(t.0.join("source/keep.so")).unwrap(),
        "preserve unrelated source"
    );
    assert_eq!(
        fs::read_to_string(output.join("arm64-v8a/libcontrolab_native.so")).unwrap(),
        "new compiled library"
    );
}
