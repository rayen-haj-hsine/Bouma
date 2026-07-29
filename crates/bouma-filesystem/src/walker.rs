//! Parallel recursive directory walking using `jwalk`.

use bouma_core::entry::{EntryKind, FileEntry};
use bouma_core::error::{BoumaError, BoumaResult};
use jwalk::WalkDir;
use std::fs;
use std::path::Path;
use tracing::debug;

/// Recursively scans `root` directory up to `max_depth` (e.g., 5 levels deep).
///
/// Uses `jwalk` for multi-threaded parallel scanning to saturate I/O.
pub fn walk_directory_recursive(root: &Path, max_depth: usize) -> BoumaResult<Vec<FileEntry>> {
    if !root.exists() {
        return Err(BoumaError::NotFound(root.to_path_buf()));
    }
    if !root.is_dir() {
        return Err(BoumaError::NotADirectory(root.to_path_buf()));
    }

    let walker = WalkDir::new(root)
        .max_depth(max_depth)
        .skip_hidden(false);

    let entries: Vec<FileEntry> = walker
        .into_iter()
        .filter_map(|result| match result {
            Ok(dir_entry) => {
                let path = dir_entry.path();
                // Skip the root path itself
                if path == root {
                    return None;
                }

                let file_type = dir_entry.file_type;
                let kind = if file_type.is_dir() {
                    EntryKind::Directory
                } else if file_type.is_symlink() {
                    EntryKind::Symlink
                } else {
                    EntryKind::File
                };

                let metadata = dir_entry.metadata().ok();
                let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                let created = metadata.as_ref().and_then(|m| m.created().ok());
                let modified = metadata.as_ref().and_then(|m| m.modified().ok());
                let readonly = metadata
                    .as_ref()
                    .map(|m| m.permissions().readonly())
                    .unwrap_or(false);

                let file_name = dir_entry.file_name().to_os_string();
                let hidden = is_hidden_os(&file_name, metadata.as_ref());

                Some(FileEntry {
                    name: file_name,
                    path,
                    kind,
                    size,
                    created,
                    modified,
                    readonly,
                    hidden,
                })
            }
            Err(_) => None,
        })
        .collect();

    debug!(
        "Recursive scan of {} found {} items (max_depth: {})",
        root.display(),
        entries.len(),
        max_depth
    );

    Ok(entries)
}

#[cfg(windows)]
fn is_hidden_os(name: &std::ffi::OsStr, metadata: Option<&fs::Metadata>) -> bool {
    use std::os::windows::fs::MetadataExt;
    if let Some(m) = metadata {
        (m.file_attributes() & 0x2) != 0 || name.to_string_lossy().starts_with('.')
    } else {
        name.to_string_lossy().starts_with('.')
    }
}

#[cfg(not(windows))]
fn is_hidden_os(name: &std::ffi::OsStr, _metadata: Option<&fs::Metadata>) -> bool {
    name.to_string_lossy().starts_with('.')
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("bouma_walk_{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_walk_recursive_finds_subfolder_files() {
        let root = test_dir("subfolder");
        let sub = root.join("level1").join("level2");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("nested.txt"), "hello").unwrap();

        let entries = walk_directory_recursive(&root, 5).unwrap();
        assert!(entries.iter().any(|e| e.display_name() == "nested.txt"));

        let _ = fs::remove_dir_all(&root);
    }
}
