//! File operations (create, rename, delete, copy, move) with progress channels.

use bouma_core::error::{BoumaError, BoumaResult};
use bouma_core::operations::{OperationKind, OperationProgress};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Buffer size for file copy chunks (128 KB for high throughput).
const COPY_BUFFER_SIZE: usize = 128 * 1024;

/// Creates a new directory inside `parent`.
pub fn create_directory(parent: &Path, name: &str) -> BoumaResult<PathBuf> {
    let name = name.trim();
    if name.is_empty() {
        return Err(BoumaError::InvalidQuery("Directory name cannot be empty".to_string()));
    }

    let target = parent.join(name);
    if target.exists() {
        return Err(BoumaError::AlreadyExists(target));
    }

    fs::create_dir_all(&target).map_err(|e| BoumaError::io(e, &target))?;
    info!("Created directory: {}", target.display());
    Ok(target)
}

/// Renames a file or directory.
pub fn rename_entry(from: &Path, new_name: &str) -> BoumaResult<PathBuf> {
    let new_name = new_name.trim();
    if new_name.is_empty() {
        return Err(BoumaError::InvalidQuery("New name cannot be empty".to_string()));
    }

    let parent = from
        .parent()
        .ok_or_else(|| BoumaError::NotFound(from.to_path_buf()))?;

    let to = parent.join(new_name);
    if to.exists() {
        return Err(BoumaError::AlreadyExists(to));
    }

    fs::rename(from, &to).map_err(|e| BoumaError::io(e, from))?;
    info!("Renamed {} -> {}", from.display(), to.display());
    Ok(to)
}

/// Deletes a file or directory by moving it to the system Recycle Bin (trash).
pub fn delete_entry(path: &Path) -> BoumaResult<()> {
    if !path.exists() {
        return Err(BoumaError::NotFound(path.to_path_buf()));
    }

    trash::delete(path).map_err(|e| {
        warn!("Failed to move {} to trash: {}", path.display(), e);
        BoumaError::Io {
            source: std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
            path: path.to_path_buf(),
        }
    })?;

    info!("Moved to Recycle Bin: {}", path.display());
    Ok(())
}

/// Copies a file or folder into `dst_dir`.
///
/// Progress updates are reported via `progress_tx` if provided.
pub fn copy_entry(
    src: &Path,
    dst_dir: &Path,
    progress_tx: Option<&std::sync::mpsc::Sender<OperationProgress>>,
) -> BoumaResult<PathBuf> {
    if !src.exists() {
        return Err(BoumaError::NotFound(src.to_path_buf()));
    }

    let file_name = src
        .file_name()
        .ok_or_else(|| BoumaError::NotFound(src.to_path_buf()))?;

    let target = dst_dir.join(file_name);
    if target.exists() {
        return Err(BoumaError::AlreadyExists(target));
    }

    if src.is_dir() {
        copy_dir_recursive(src, &target, progress_tx)?;
    } else {
        copy_single_file(src, &target, progress_tx)?;
    }

    info!("Copied {} -> {}", src.display(), target.display());
    Ok(target)
}

/// Moves a file or folder into `dst_dir`.
pub fn move_entry(
    src: &Path,
    dst_dir: &Path,
    progress_tx: Option<&std::sync::mpsc::Sender<OperationProgress>>,
) -> BoumaResult<PathBuf> {
    if !src.exists() {
        return Err(BoumaError::NotFound(src.to_path_buf()));
    }

    let file_name = src
        .file_name()
        .ok_or_else(|| BoumaError::NotFound(src.to_path_buf()))?;

    let target = dst_dir.join(file_name);
    if target.exists() {
        return Err(BoumaError::AlreadyExists(target));
    }

    // Try atomic fast-rename first (works on same drive/volume)
    if let Ok(()) = fs::rename(src, &target) {
        info!("Moved (fast) {} -> {}", src.display(), target.display());
        return Ok(target);
    }

    // Fallback across volumes: copy then delete
    copy_entry(src, dst_dir, progress_tx)?;
    if src.is_dir() {
        fs::remove_dir_all(src).map_err(|e| BoumaError::io(e, src))?;
    } else {
        fs::remove_file(src).map_err(|e| BoumaError::io(e, src))?;
    }

    info!("Moved (copy+delete) {} -> {}", src.display(), target.display());
    Ok(target)
}

/// Copies a single file with chunked buffer reading and progress updates.
fn copy_single_file(
    src: &Path,
    dst: &Path,
    progress_tx: Option<&std::sync::mpsc::Sender<OperationProgress>>,
) -> BoumaResult<()> {
    let mut reader = fs::File::open(src).map_err(|e| BoumaError::io(e, src))?;
    let mut writer = fs::File::create(dst).map_err(|e| BoumaError::io(e, dst))?;

    let total_bytes = reader.metadata().map(|m| m.len()).unwrap_or(0);
    let mut bytes_done = 0u64;
    let mut buffer = vec![0u8; COPY_BUFFER_SIZE];

    loop {
        let n = reader.read(&mut buffer).map_err(|e| BoumaError::io(e, src))?;
        if n == 0 {
            break;
        }

        writer.write_all(&buffer[..n]).map_err(|e| BoumaError::io(e, dst))?;
        bytes_done += n as u64;

        if let Some(tx) = progress_tx {
            let progress = OperationProgress {
                kind: OperationKind::Copy,
                source: src.to_path_buf(),
                destination: Some(dst.to_path_buf()),
                total_bytes,
                bytes_done,
                total_items: 1,
                items_done: if bytes_done >= total_bytes { 1 } else { 0 },
            };
            let _ = tx.send(progress);
        }
    }

    writer.flush().map_err(|e| BoumaError::io(e, dst))?;
    Ok(())
}

/// Recursively copies a directory.
fn copy_dir_recursive(
    src: &Path,
    dst: &Path,
    progress_tx: Option<&std::sync::mpsc::Sender<OperationProgress>>,
) -> BoumaResult<()> {
    fs::create_dir_all(dst).map_err(|e| BoumaError::io(e, dst))?;

    for entry in fs::read_dir(src).map_err(|e| BoumaError::io(e, src))? {
        let entry = entry.map_err(|e| BoumaError::io(e, src))?;
        let entry_path = entry.path();
        let target_path = dst.join(entry.file_name());

        if entry_path.is_dir() {
            copy_dir_recursive(&entry_path, &target_path, progress_tx)?;
        } else {
            copy_single_file(&entry_path, &target_path, progress_tx)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("bouma_ops_{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_create_and_rename_directory() {
        let parent = test_dir("create_rename");
        let created = create_directory(&parent, "new_folder").unwrap();
        assert!(created.exists());

        let renamed = rename_entry(&created, "renamed_folder").unwrap();
        assert!(!created.exists());
        assert!(renamed.exists());

        let _ = fs::remove_dir_all(&parent);
    }

    #[test]
    fn test_copy_file() {
        let root = test_dir("copy_file");
        let src_file = root.join("src.txt");
        fs::write(&src_file, "content data").unwrap();

        let dst_dir = root.join("dst_folder");
        fs::create_dir(&dst_dir).unwrap();

        let copied = copy_entry(&src_file, &dst_dir, None).unwrap();
        assert!(copied.exists());
        assert_eq!(fs::read_to_string(copied).unwrap(), "content data");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_move_file() {
        let root = test_dir("move_file");
        let src_file = root.join("move_src.txt");
        fs::write(&src_file, "move me").unwrap();

        let dst_dir = root.join("dst_dir");
        fs::create_dir(&dst_dir).unwrap();

        let moved = move_entry(&src_file, &dst_dir, None).unwrap();
        assert!(!src_file.exists());
        assert!(moved.exists());
        assert_eq!(fs::read_to_string(moved).unwrap(), "move me");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_create_duplicate_error() {
        let root = test_dir("dup_error");
        create_directory(&root, "sub").unwrap();
        let err = create_directory(&root, "sub");
        assert!(matches!(err, Err(BoumaError::AlreadyExists(_))));
        let _ = fs::remove_dir_all(&root);
    }
}
