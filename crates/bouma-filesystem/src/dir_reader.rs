//! Directory reading with parallel metadata collection.
//!
//! The strategy:
//! 1. `std::fs::read_dir` to get directory entries (fast, single-threaded)
//! 2. `rayon::par_iter` to collect metadata for all entries in parallel
//!    (this saturates I/O on large directories)
//! 3. Return sorted `Vec<FileEntry>`

use bouma_core::entry::{EntryKind, FileEntry};
use bouma_core::error::{BoumaError, BoumaResult};
use rayon::prelude::*;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::time::Instant;
use tracing::{debug, warn};

/// Reads a directory and returns all entries with full metadata.
///
/// Entries are collected in parallel using `rayon` for fast metadata reads.
/// Hidden files and system files are included — filtering is the UI's job.
///
/// # Errors
///
/// Returns `BoumaError::NotFound` if the path doesn't exist,
/// `BoumaError::NotADirectory` if it's not a directory, or
/// `BoumaError::PermissionDenied` if access is denied.
pub fn read_directory(path: &Path) -> BoumaResult<Vec<FileEntry>> {
    let start = Instant::now();

    // Validate path
    if !path.exists() {
        return Err(BoumaError::NotFound(path.to_path_buf()));
    }
    if !path.is_dir() {
        return Err(BoumaError::NotADirectory(path.to_path_buf()));
    }

    // Phase 1: Read directory entries (single-threaded, fast)
    let read_start = Instant::now();
    let raw_entries: Vec<_> = fs::read_dir(path)
        .map_err(|e| BoumaError::io(e, path))?
        .filter_map(|entry| match entry {
            Ok(e) => Some(e),
            Err(err) => {
                warn!("Skipping unreadable entry in {}: {}", path.display(), err);
                None
            }
        })
        .collect();
    let read_duration = read_start.elapsed();

    // Phase 2: Collect metadata in parallel (rayon)
    let meta_start = Instant::now();
    let entries: Vec<FileEntry> = raw_entries
        .par_iter()
        .filter_map(|dir_entry| {
            let entry_path = dir_entry.path();
            match dir_entry.metadata() {
                Ok(metadata) => {
                    let kind = if metadata.is_dir() {
                        EntryKind::Directory
                    } else if metadata.is_symlink() {
                        EntryKind::Symlink
                    } else {
                        EntryKind::File
                    };

                    let name = dir_entry.file_name();
                    let hidden = is_hidden(&name, &metadata);

                    Some(FileEntry {
                        name,
                        path: entry_path,
                        kind,
                        size: if metadata.is_file() { metadata.len() } else { 0 },
                        created: metadata.created().ok(),
                        modified: metadata.modified().ok(),
                        readonly: metadata.permissions().readonly(),
                        hidden,
                    })
                }
                Err(err) => {
                    warn!(
                        "Failed to read metadata for {}: {}",
                        entry_path.display(),
                        err
                    );
                    None
                }
            }
        })
        .collect();
    let meta_duration = meta_start.elapsed();

    debug!(
        "Read {} entries from {} in {:?} (read: {:?}, metadata: {:?})",
        entries.len(),
        path.display(),
        start.elapsed(),
        read_duration,
        meta_duration,
    );

    Ok(entries)
}

/// Checks if a file is hidden.
///
/// On Windows, this checks the file attributes for the hidden flag.
/// On other platforms, checks for dot-prefix.
#[cfg(windows)]
fn is_hidden(name: &OsString, metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    // FILE_ATTRIBUTE_HIDDEN = 0x2
    let attrs = metadata.file_attributes();
    (attrs & 0x2) != 0 || name.to_string_lossy().starts_with('.')
}

#[cfg(not(windows))]
fn is_hidden(name: &OsString, _metadata: &fs::Metadata) -> bool {
    name.to_string_lossy().starts_with('.')
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("bouma_test_{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_read_empty_directory() {
        let dir = test_dir("read_empty_dir");
        let entries = read_directory(&dir).unwrap();
        assert!(entries.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_read_directory_with_files() {
        let dir = test_dir("read_dir_files");
        fs::write(dir.join("file1.txt"), "hello").unwrap();
        fs::write(dir.join("file2.txt"), "world").unwrap();
        fs::create_dir(dir.join("subdir")).unwrap();

        let entries = read_directory(&dir).unwrap();
        assert_eq!(entries.len(), 3);

        let names: Vec<String> = entries.iter().map(|e| e.display_name()).collect();
        assert!(names.contains(&"file1.txt".to_string()));
        assert!(names.contains(&"file2.txt".to_string()));
        assert!(names.contains(&"subdir".to_string()));

        // Check types
        let subdir = entries.iter().find(|e| e.display_name() == "subdir").unwrap();
        assert_eq!(subdir.kind, EntryKind::Directory);

        let file1 = entries.iter().find(|e| e.display_name() == "file1.txt").unwrap();
        assert_eq!(file1.kind, EntryKind::File);
        assert_eq!(file1.size, 5); // "hello" = 5 bytes

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_read_nonexistent_directory() {
        let result = read_directory(Path::new("C:\\this_does_not_exist_bouma_test"));
        assert!(matches!(result, Err(BoumaError::NotFound(_))));
    }

    #[test]
    fn test_read_file_as_directory() {
        let dir = test_dir("read_file_as_dir");
        let file_path = dir.join("not_a_dir.txt");
        fs::write(&file_path, "content").unwrap();

        let result = read_directory(&file_path);
        assert!(matches!(result, Err(BoumaError::NotADirectory(_))));

        let _ = fs::remove_dir_all(&dir);
    }
}
