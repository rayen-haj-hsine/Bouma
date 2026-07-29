//! Sorting logic for file entries.

use crate::entry::{EntryKind, FileEntry};
use serde::{Deserialize, Serialize};

/// Which field to sort file entries by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SortField {
    Name,
    Size,
    Modified,
    Created,
    Kind,
}

/// Sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl SortOrder {
    /// Toggles between ascending and descending.
    pub fn toggle(self) -> Self {
        match self {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        }
    }
}

/// Sorts a slice of file entries in place.
///
/// **Directories always come first**, regardless of the sort field.
/// Within directories and within files, entries are sorted by the specified
/// field and order.
pub fn sort_entries(entries: &mut [FileEntry], field: SortField, order: SortOrder) {
    entries.sort_by(|a, b| {
        // Directories first, always.
        let dir_cmp = is_directory(b).cmp(&is_directory(a));
        if dir_cmp != std::cmp::Ordering::Equal {
            return dir_cmp;
        }

        let cmp = match field {
            SortField::Name => compare_names(a, b),
            SortField::Size => a.size.cmp(&b.size),
            SortField::Modified => compare_times(a.modified, b.modified),
            SortField::Created => compare_times(a.created, b.created),
            SortField::Kind => compare_kinds(a, b),
        };

        match order {
            SortOrder::Ascending => cmp,
            SortOrder::Descending => cmp.reverse(),
        }
    });
}

fn is_directory(entry: &FileEntry) -> bool {
    entry.kind == EntryKind::Directory
}

/// Case-insensitive natural name comparison.
fn compare_names(a: &FileEntry, b: &FileEntry) -> std::cmp::Ordering {
    let a_name = a.display_name().to_lowercase();
    let b_name = b.display_name().to_lowercase();
    a_name.cmp(&b_name)
}

fn compare_times(
    a: Option<std::time::SystemTime>,
    b: Option<std::time::SystemTime>,
) -> std::cmp::Ordering {
    match (a, b) {
        (Some(a), Some(b)) => a.cmp(&b),
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

fn compare_kinds(a: &FileEntry, b: &FileEntry) -> std::cmp::Ordering {
    let a_ext = a.extension().unwrap_or_default();
    let b_ext = b.extension().unwrap_or_default();
    a_ext.cmp(&b_ext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::path::PathBuf;

    fn make_entry(name: &str, kind: EntryKind, size: u64) -> FileEntry {
        FileEntry {
            name: OsString::from(name),
            path: PathBuf::from(format!("C:\\{name}")),
            kind,
            size,
            created: None,
            modified: None,
            readonly: false,
            hidden: false,
        }
    }

    #[test]
    fn test_directories_come_first() {
        let mut entries = vec![
            make_entry("file_b.txt", EntryKind::File, 200),
            make_entry("dir_a", EntryKind::Directory, 0),
            make_entry("file_a.txt", EntryKind::File, 100),
            make_entry("dir_b", EntryKind::Directory, 0),
        ];

        sort_entries(&mut entries, SortField::Name, SortOrder::Ascending);

        assert_eq!(entries[0].display_name(), "dir_a");
        assert_eq!(entries[1].display_name(), "dir_b");
        assert_eq!(entries[2].display_name(), "file_a.txt");
        assert_eq!(entries[3].display_name(), "file_b.txt");
    }

    #[test]
    fn test_sort_by_size() {
        let mut entries = vec![
            make_entry("big.txt", EntryKind::File, 1000),
            make_entry("small.txt", EntryKind::File, 10),
            make_entry("medium.txt", EntryKind::File, 500),
        ];

        sort_entries(&mut entries, SortField::Size, SortOrder::Ascending);

        assert_eq!(entries[0].size, 10);
        assert_eq!(entries[1].size, 500);
        assert_eq!(entries[2].size, 1000);
    }

    #[test]
    fn test_sort_descending() {
        let mut entries = vec![
            make_entry("a.txt", EntryKind::File, 10),
            make_entry("c.txt", EntryKind::File, 30),
            make_entry("b.txt", EntryKind::File, 20),
        ];

        sort_entries(&mut entries, SortField::Name, SortOrder::Descending);

        assert_eq!(entries[0].display_name(), "c.txt");
        assert_eq!(entries[1].display_name(), "b.txt");
        assert_eq!(entries[2].display_name(), "a.txt");
    }

    #[test]
    fn test_case_insensitive_sort() {
        let mut entries = vec![
            make_entry("Zebra.txt", EntryKind::File, 0),
            make_entry("apple.txt", EntryKind::File, 0),
            make_entry("Banana.txt", EntryKind::File, 0),
        ];

        sort_entries(&mut entries, SortField::Name, SortOrder::Ascending);

        assert_eq!(entries[0].display_name(), "apple.txt");
        assert_eq!(entries[1].display_name(), "Banana.txt");
        assert_eq!(entries[2].display_name(), "Zebra.txt");
    }

    #[test]
    fn test_toggle_sort_order() {
        assert_eq!(SortOrder::Ascending.toggle(), SortOrder::Descending);
        assert_eq!(SortOrder::Descending.toggle(), SortOrder::Ascending);
    }
}
