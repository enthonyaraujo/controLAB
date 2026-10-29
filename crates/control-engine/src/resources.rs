//! Shared asset discovery for source checkouts and packaged Rust engines.
use crate::Result;
use std::{
    env, fs,
    path::{Component, Path, PathBuf},
};

/// Resolve the shared asset tree. An explicit override never falls back.
pub fn assets_root() -> Result<PathBuf> {
    let override_path = env::var_os("CONTROLAB_ASSETS").filter(|v| !v.is_empty());
    if let Some(path) = override_path {
        return verified_root(&expand_home(Path::new(&path))).map_err(|_| {
            format!(
                "Recursos do ControLAB ausentes em {}",
                expand_home(Path::new(&path)).display()
            )
        });
    }
    if let Ok(exe) = env::current_exe()
        && let Some(parent) = exe.parent()
    {
        // The distributed engine/ directory is an explicit bundle layout:
        // an incomplete bundle must not accidentally use a developer checkout.
        if parent.file_name().is_some_and(|name| name == "engine") {
            return verified_root(&parent.join("../resources/assets"))
                .map_err(|_| "Pacote nativo sem os recursos compartilhados assets/".into());
        }
        for candidate in [
            parent.join("../resources/assets"),
            parent.join("resources/assets"),
            parent.join("assets"),
        ] {
            if candidate.join("manifest.json").is_file() {
                return verified_root(&candidate);
            }
        }
    }
    verified_root(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets"))
        .map_err(|_| "Recursos do ControLAB ausentes: configure CONTROLAB_ASSETS".into())
}

fn verified_root(path: &Path) -> Result<PathBuf> {
    let root = fs::canonicalize(path).map_err(|e| e.to_string())?;
    if !root.join("manifest.json").is_file() {
        return Err(format!(
            "Recursos do ControLAB ausentes em {}",
            root.display()
        ));
    }
    Ok(root)
}

fn expand_home(path: &Path) -> PathBuf {
    if let Some(home) = env::var_os("HOME").or_else(|| env::var_os("USERPROFILE")) {
        if path == Path::new("~") {
            return home.into();
        }
        if let Ok(relative) = path.strip_prefix("~") {
            return PathBuf::from(home).join(relative);
        }
    }
    path.into()
}

/// Resolve symlinks even when the final resource does not exist yet. Never
/// permit an absolute component or a traversal to escape the canonical tree.
pub fn asset_path(parts: &[&str]) -> Result<PathBuf> {
    let root = assets_root()?;
    let mut joined = root.clone();
    for part in parts {
        joined.push(part);
    }
    let mut resolved = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                resolved.pop();
            }
            _ => {
                resolved.push(component.as_os_str());
                match fs::symlink_metadata(&resolved) {
                    Ok(_) => resolved = fs::canonicalize(&resolved).map_err(|e| e.to_string())?,
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => return Err(e.to_string()),
                }
            }
        }
    }
    if !resolved.starts_with(&root) {
        return Err("O recurso deve permanecer dentro de assets/".into());
    }
    Ok(resolved)
}
