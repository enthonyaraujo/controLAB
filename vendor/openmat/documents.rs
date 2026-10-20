use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

pub const MAX_SCRIPT_BYTES: usize = 1_048_576;

pub struct Document {
    pub id: u64,
    pub path: Option<PathBuf>,
    pub source: String,
    saved_source: String,
}

impl Document {
    pub fn untitled(id: u64, source: String) -> Self {
        Self {
            id,
            path: None,
            source,
            saved_source: String::new(),
        }
    }
    pub fn open(id: u64, path: &Path) -> io::Result<Self> {
        let path = path.canonicalize()?;
        ensure_script(&path)?;
        let mut source = String::new();
        fs::File::open(&path)?
            .take((MAX_SCRIPT_BYTES + 1) as u64)
            .read_to_string(&mut source)?;
        if source.len() > MAX_SCRIPT_BYTES {
            return Err(io::Error::other("script excede 1 MiB"));
        }
        Ok(Self {
            id,
            path: Some(path),
            saved_source: source.clone(),
            source,
        })
    }
    pub fn dirty(&self) -> bool {
        self.source != self.saved_source
    }
    pub fn title(&self) -> String {
        self.path.as_ref().and_then(|p| p.file_name()).map_or_else(
            || format!("Sem título {}.m", self.id),
            |s| s.to_string_lossy().into_owned(),
        )
    }
    /// Atomic sibling-file replacement. Failed saves leave the document dirty.
    pub fn save(&mut self, path: &Path) -> io::Result<()> {
        ensure_script(path)?;
        if self.source.len() > MAX_SCRIPT_BYTES {
            return Err(io::Error::other("script excede 1 MiB"));
        }
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let parent = parent.canonicalize()?;
        let name = path
            .file_name()
            .ok_or_else(|| io::Error::other("nome de arquivo inválido"))?;
        let target = parent.join(name);
        let temp = parent.join(format!(".openmat-{}-{}.tmp", std::process::id(), self.id));
        let mut created = false;
        let result = (|| {
            let mut file = fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&temp)?;
            created = true;
            file.write_all(self.source.as_bytes())?;
            file.sync_all()?;
            // Preserve existing permissions when replacing an edited script.
            if let Ok(metadata) = fs::metadata(&target) {
                fs::set_permissions(&temp, metadata.permissions())?;
            }
            fs::rename(&temp, &target)
        })();
        if result.is_err() && created {
            let _ = fs::remove_file(&temp);
        }
        result?;
        self.path = Some(target);
        self.saved_source.clone_from(&self.source);
        Ok(())
    }
}

fn ensure_script(path: &Path) -> io::Result<()> {
    if path
        .extension()
        .is_none_or(|ext| !ext.eq_ignore_ascii_case("m"))
    {
        return Err(io::Error::other("selecione um arquivo .m"));
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct FolderEntry {
    pub path: PathBuf,
    pub directory: bool,
}

pub fn list_folder(path: &Path) -> io::Result<Vec<FolderEntry>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        let directory = path.is_dir();
        if directory
            || path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("m"))
        {
            entries.push(FolderEntry { path, directory });
        }
    }
    entries.sort_by(|a, b| {
        b.directory
            .cmp(&a.directory)
            .then_with(|| a.path.cmp(&b.path))
    });
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    fn fixture() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "openmat-doc-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        path
    }
    #[test]
    fn open_edit_save_roundtrip_tracks_unsaved_state() {
        let folder = fixture();
        let path = folder.join("test.m");
        fs::write(&path, "A=[1 2;3 4];\n").unwrap();
        let mut doc = Document::open(1, &path).unwrap();
        assert!(!doc.dirty());
        doc.source.push_str("det(A)\n");
        assert!(doc.dirty());
        doc.save(&path).unwrap();
        assert!(!doc.dirty());
        assert_eq!(fs::read_to_string(path).unwrap(), doc.source);
        fs::remove_dir_all(folder).unwrap();
    }
    #[test]
    fn failed_save_preserves_original_and_dirty_document() {
        let folder = fixture();
        let path = folder.join("test.m");
        fs::write(&path, "original").unwrap();
        let mut doc = Document::open(1, &path).unwrap();
        doc.source = "edited".into();
        assert!(doc.save(&folder.join("missing/test.m")).is_err());
        assert!(doc.dirty());
        assert_eq!(fs::read_to_string(path).unwrap(), "original");
        fs::remove_dir_all(folder).unwrap();
    }
    #[test]
    fn folder_lists_only_directories_and_scripts() {
        let folder = fixture();
        fs::create_dir(folder.join("sub")).unwrap();
        fs::write(folder.join("a.m"), "1").unwrap();
        fs::write(folder.join("data.txt"), "1").unwrap();
        let entries = list_folder(&folder).unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries[0].directory);
        assert!(Document::open(1, &folder.join("data.txt")).is_err());
        fs::remove_dir_all(folder).unwrap();
    }
    #[test]
    fn temporary_file_collision_does_not_remove_an_existing_file() {
        let folder = fixture();
        let temp = folder.join(format!(".openmat-{}-21.tmp", std::process::id()));
        fs::write(&temp, "preservar").unwrap();
        let mut doc = Document::untitled(21, "A=1;".into());
        assert!(doc.save(&folder.join("script.m")).is_err());
        assert_eq!(fs::read_to_string(&temp).unwrap(), "preservar");
        assert!(doc.dirty());
        fs::remove_dir_all(folder).unwrap();
    }
}
