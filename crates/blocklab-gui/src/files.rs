//! Ler e gravar arquivos, e os diálogos nativos de abrir/salvar.
//!
//! No Android não há diálogo nativo: tudo fica na pasta do app
//! ([`set_data_dir`]), e "Abrir…" mostra a lista de [`saved_files`].

use std::path::{Path, PathBuf};

use blocklab_core::diagram::Diagram;

pub fn load(path: &Path) -> Result<Diagram, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Diagram::from_json(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Grava de forma segura: escreve num arquivo temporário ao lado e
/// renomeia. Se faltar disco ou o programa cair no meio, o arquivo antigo
/// continua intacto (renomear é atômico no mesmo sistema de arquivos).
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    std::fs::write(&tmp, bytes).map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("{}: {e}", path.display())
    })
}

pub fn save(path: &Path, d: &Diagram) -> Result<(), String> {
    write_atomic(path, d.to_json().as_bytes())
}

/// Garante a extensão (o diálogo do GTK não acrescenta sozinho).
pub fn with_extension(path: PathBuf, ext: &str) -> PathBuf {
    let has = path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(ext));
    if has {
        path
    } else {
        let mut s = path.into_os_string();
        s.push(".");
        s.push(ext);
        PathBuf::from(s)
    }
}

#[cfg(target_os = "android")]
static DATA_DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

/// Pasta do app no Android (definida ao iniciar).
#[cfg(target_os = "android")]
pub fn set_data_dir(dir: PathBuf) {
    let _ = std::fs::create_dir_all(&dir);
    let _ = DATA_DIR.set(dir);
}

#[cfg(target_os = "android")]
pub fn data_dir() -> Option<&'static Path> {
    DATA_DIR.get().map(PathBuf::as_path)
}

/// Diagramas na pasta do app, do mais recente ao mais antigo.
#[cfg(target_os = "android")]
pub fn saved_files() -> Vec<PathBuf> {
    let Some(entries) = data_dir().and_then(|d| std::fs::read_dir(d).ok()) else {
        return Vec::new();
    };
    let mut found: Vec<(std::time::SystemTime, PathBuf)> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("json"))
        })
        .map(|p| {
            let t = std::fs::metadata(&p)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::UNIX_EPOCH);
            (t, p)
        })
        .collect();
    found.sort_by_key(|(t, _)| std::cmp::Reverse(*t));
    found.into_iter().map(|(_, p)| p).collect()
}

/// No Android a lista de arquivos é uma janela do próprio app
/// (`App::browse`), então aqui não há o que escolher.
#[cfg(target_os = "android")]
pub fn pick_open() -> Option<PathBuf> {
    None
}

/// No Android: o nome sugerido na pasta do app, sem sobrescrever outro
/// arquivo ("diagrama-2.json", "diagrama-3.json"...).
#[cfg(target_os = "android")]
pub fn pick_save(_title: &str, _filter: &str, ext: &str, suggested: &str) -> Option<PathBuf> {
    let dir = data_dir()?;
    let mut path = dir.join(format!("{suggested}.{ext}"));
    let mut k = 2;
    while path.exists() {
        path = dir.join(format!("{suggested}-{k}.{ext}"));
        k += 1;
    }
    Some(path)
}

#[cfg(not(target_os = "android"))]
pub fn pick_open() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Abrir diagrama")
        .add_filter("Diagrama ControLAB (JSON)", &["json"])
        .pick_file()
}

/// Diálogo de salvar com nome sugerido; devolve o caminho já com a
/// extensão certa.
#[cfg(not(target_os = "android"))]
pub fn pick_save(title: &str, filter: &str, ext: &str, suggested: &str) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title(title)
        .add_filter(filter, &[ext])
        .set_file_name(format!("{suggested}.{ext}"))
        .save_file()
        .map(|p| with_extension(p, ext))
}

#[cfg(test)]
mod tests {
    use super::*;
    use blocklab_core::diagram::{Sign, examples};

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("blocklab-teste-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    #[test]
    fn salvar_e_abrir() {
        let p = temp("ref.json");
        let mut d = examples::reference(Sign::Minus);
        d.params.insert("G1".into(), "10".into());
        save(&p, &d).unwrap();
        assert_eq!(load(&p).unwrap(), d);
        // Não sobra arquivo temporário.
        assert!(!p.with_extension("json.tmp").exists());
    }

    #[test]
    fn erro_de_leitura_diz_o_arquivo() {
        let p = temp("quebrado.json");
        std::fs::write(&p, "{ isso não é json").unwrap();
        let err = load(&p).unwrap_err();
        assert!(
            err.contains("quebrado.json") && err.contains("JSON inválido"),
            "{err}"
        );
        assert!(load(&temp("nao-existe.json")).is_err());
    }

    #[test]
    fn extensao_e_acrescentada() {
        assert_eq!(
            with_extension("a/b".into(), "json"),
            PathBuf::from("a/b.json")
        );
        assert_eq!(
            with_extension("a/b.JSON".into(), "json"),
            PathBuf::from("a/b.JSON")
        );
        assert_eq!(
            with_extension("fig.v2".into(), "svg"),
            PathBuf::from("fig.v2.svg")
        );
    }
}
