//! File and directory entry domain types.

use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::SystemTime;

/// The kind of filesystem entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntryKind {
    /// A regular file.
    File,
    /// A directory.
    Directory,
    /// A symbolic link.
    Symlink,
}

impl EntryKind {
    /// Returns a human-readable label for this entry kind.
    pub fn label(&self) -> &'static str {
        match self {
            EntryKind::File => "File",
            EntryKind::Directory => "Folder",
            EntryKind::Symlink => "Symlink",
        }
    }
}

/// Canonical representation of a filesystem entry (file, directory, or symlink).
///
/// This is the core domain type that flows through every layer of Bouma.
/// It is intentionally lightweight — only the metadata needed for display
/// and operations is included.
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// The file/directory name (without parent path).
    pub name: OsString,

    /// The full absolute path.
    pub path: PathBuf,

    /// What kind of entry this is.
    pub kind: EntryKind,

    /// Size in bytes. `0` for directories (size-on-disk is computed separately).
    pub size: u64,

    /// When the entry was created. `None` if the filesystem doesn't support it.
    pub created: Option<SystemTime>,

    /// When the entry was last modified.
    pub modified: Option<SystemTime>,

    /// Whether the entry is read-only.
    pub readonly: bool,

    /// Whether the entry is hidden (Windows hidden attribute or dot-prefixed name).
    pub hidden: bool,
}

impl FileEntry {
    /// Returns the file extension as a lowercase string, or `None` for directories
    /// and files without extensions.
    pub fn extension(&self) -> Option<String> {
        if self.kind == EntryKind::Directory {
            return None;
        }
        self.path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
    }

    /// Returns the display name as a UTF-8 string, lossy-converting if necessary.
    pub fn display_name(&self) -> String {
        self.name.to_string_lossy().into_owned()
    }

    /// Returns a human-readable size string (e.g., "4.2 MB").
    pub fn display_size(&self) -> String {
        if self.kind == EntryKind::Directory {
            return String::from("—");
        }
        format_size(self.size)
    }
}

/// Formats a byte count into a human-readable string.
///
/// Uses binary prefixes (KiB, MiB, etc.) but labels them with common
/// abbreviations (KB, MB) for user familiarity.
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    const TB: u64 = 1024 * GB;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1048576), "1.0 MB");
        assert_eq!(format_size(1073741824), "1.0 GB");
        assert_eq!(format_size(1099511627776), "1.0 TB");
    }

    #[test]
    fn test_entry_extension() {
        let entry = FileEntry {
            name: OsString::from("test.PDF"),
            path: PathBuf::from("C:\\test.PDF"),
            kind: EntryKind::File,
            size: 100,
            created: None,
            modified: None,
            readonly: false,
            hidden: false,
        };
        assert_eq!(entry.extension(), Some("pdf".to_string()));
    }

    #[test]
    fn test_directory_has_no_extension() {
        let entry = FileEntry {
            name: OsString::from("folder.d"),
            path: PathBuf::from("C:\\folder.d"),
            kind: EntryKind::Directory,
            size: 0,
            created: None,
            modified: None,
            readonly: false,
            hidden: false,
        };
        assert_eq!(entry.extension(), None);
    }

    #[test]
    fn test_display_size_for_directory() {
        let entry = FileEntry {
            name: OsString::from("docs"),
            path: PathBuf::from("C:\\docs"),
            kind: EntryKind::Directory,
            size: 0,
            created: None,
            modified: None,
            readonly: false,
            hidden: false,
        };
        assert_eq!(entry.display_size(), "—");
    }
}
