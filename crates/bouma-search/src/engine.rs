//! Search engine — filters a list of `FileEntry` by a `SearchQuery`.

use bouma_core::entry::FileEntry;

use crate::query::SearchQuery;

/// Filters entries that match the given search query.
///
/// Returns a new `Vec` containing only the matching entries (cloned).
/// The order of the input is preserved.
pub fn search(entries: &[FileEntry], query: &SearchQuery) -> Vec<FileEntry> {
    entries
        .iter()
        .filter(|entry| query.matches(&entry.display_name()))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bouma_core::entry::EntryKind;
    use std::ffi::OsString;
    use std::path::PathBuf;

    fn make_entry(name: &str) -> FileEntry {
        FileEntry {
            name: OsString::from(name),
            path: PathBuf::from(format!("C:\\{name}")),
            kind: EntryKind::File,
            size: 100,
            created: None,
            modified: None,
            readonly: false,
            hidden: false,
        }
    }

    #[test]
    fn test_search_substring() {
        let entries = vec![
            make_entry("report.pdf"),
            make_entry("notes.txt"),
            make_entry("final_report.pdf"),
        ];

        let query = SearchQuery::parse("report").unwrap();
        let results = search(&entries, &query);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].display_name(), "report.pdf");
        assert_eq!(results[1].display_name(), "final_report.pdf");
    }

    #[test]
    fn test_search_glob() {
        let entries = vec![
            make_entry("doc.pdf"),
            make_entry("photo.jpg"),
            make_entry("invoice.pdf"),
        ];

        let query = SearchQuery::parse("*.pdf").unwrap();
        let results = search(&entries, &query);

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_no_results() {
        let entries = vec![make_entry("file.txt")];
        let query = SearchQuery::parse("xyz").unwrap();
        let results = search(&entries, &query);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_preserves_order() {
        let entries = vec![
            make_entry("c_file.rs"),
            make_entry("a_file.rs"),
            make_entry("b_file.rs"),
        ];

        let query = SearchQuery::parse("ext:rs").unwrap();
        let results = search(&entries, &query);

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].display_name(), "c_file.rs");
        assert_eq!(results[1].display_name(), "a_file.rs");
        assert_eq!(results[2].display_name(), "b_file.rs");
    }
}
