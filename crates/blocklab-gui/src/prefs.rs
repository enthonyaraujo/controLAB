//! Preferências guardadas entre sessões: tema e arquivos recentes.
//!
//! Arquivo de texto simples, uma chave por linha:
//!
//! ```text
//! tema=escuro
//! recente=/home/ana/diagramas/malha.json
//! ```

use std::path::{Path, PathBuf};

use eframe::egui::ThemePreference;

use crate::theme;

/// Quantos arquivos recentes lembrar.
pub const MAX_RECENT: usize = 6;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Prefs {
    pub theme: ThemePreference,
    pub recent: Vec<PathBuf>,
}

impl Prefs {
    pub fn parse(text: &str) -> Self {
        let mut p = Prefs::default();
        for line in text.lines() {
            match line.split_once('=') {
                Some(("tema", v)) => {
                    if let Some(t) = theme::parse_preference(v) {
                        p.theme = t;
                    }
                }
                Some(("recente", v)) if !v.trim().is_empty() && p.recent.len() < MAX_RECENT => {
                    p.recent.push(PathBuf::from(v.trim()));
                }
                _ => {}
            }
        }
        p
    }

    pub fn to_text(&self) -> String {
        let mut s = format!("tema={}\n", theme::preference_name(self.theme));
        for r in &self.recent {
            s.push_str(&format!("recente={}\n", r.display()));
        }
        s
    }

    /// Põe `path` no topo dos recentes (sem repetir).
    pub fn touch(&mut self, path: &Path) {
        self.recent.retain(|p| p != path);
        self.recent.insert(0, path.to_path_buf());
        self.recent.truncate(MAX_RECENT);
    }

    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .map(|s| Prefs::parse(&s))
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        crate::files::write_atomic(path, self.to_text().as_bytes())
    }
}

/// Onde o sistema guarda configurações de programas (no Android, a pasta
/// do app).
#[cfg(target_os = "android")]
pub fn default_path() -> Option<PathBuf> {
    crate::files::data_dir().map(|d| d.join("preferencias.txt"))
}

/// Onde o sistema guarda configurações de programas.
#[cfg(not(target_os = "android"))]
pub fn default_path() -> Option<PathBuf> {
    let env = |k: &str| {
        std::env::var_os(k)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    let base = if cfg!(windows) {
        env("APPDATA")
    } else if cfg!(target_os = "macos") {
        env("HOME").map(|h| h.join("Library/Application Support"))
    } else {
        env("XDG_CONFIG_HOME").or_else(|| env("HOME").map(|h| h.join(".config")))
    }?;
    Some(
        base.join("controlab-native")
            .join("diagrams")
            .join("preferences.txt"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ida_e_volta() {
        let mut p = Prefs {
            theme: ThemePreference::Dark,
            recent: vec![],
        };
        p.touch(Path::new("/a.json"));
        p.touch(Path::new("/b.json"));
        p.touch(Path::new("/a.json"));
        assert_eq!(
            p.recent,
            [PathBuf::from("/a.json"), PathBuf::from("/b.json")]
        );
        assert_eq!(Prefs::parse(&p.to_text()), p);
    }

    #[test]
    fn ignora_linhas_estranhas() {
        let p = Prefs::parse("tema=roxo\nlixo\nrecente=\nrecente=/x.json\n");
        assert_eq!(p.theme, ThemePreference::System);
        assert_eq!(p.recent, [PathBuf::from("/x.json")]);
    }
}
