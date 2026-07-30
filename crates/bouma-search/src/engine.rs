//! Search engine — filters and ranks a list of `FileEntry` by a `SearchQuery`.
//!
//! Results are sorted by relevance, not just filtered. Relevance tiers (ascending = worse):
//!
//! | Score | Rule                              | Example query "id"        |
//! |-------|-----------------------------------|---------------------------|
//! |   0   | Stem is **exactly** the query     | `id.txt`, `ID.pdf`        |
//! |   1   | Stem **starts with** the query    | `identity.doc`, `id_card` |
//! |   2   | Query is a **whole word** in stem | `user_id.txt`, `emp-id`   |
//! |   3   | Stem **contains** query anywhere  | `video`, `audio`, `valid` |
//!
//! All matching is **case-insensitive**. Within each tier, files rank before directories.

use bouma_core::entry::{EntryKind, FileEntry, FileTypeFilter};

use crate::query::{QueryMatcher, SearchQuery};

/// Like [`search`] but returns `(relevance_score, entry)` pairs so callers can group results
/// by tier. Score 0 = best (exact match), 3 = worst (loose substring).
pub fn search_scored(
    entries: &[FileEntry],
    query: &SearchQuery,
    type_filter: FileTypeFilter,
) -> Vec<(u8, FileEntry)> {
    if !matches!(query.matcher, QueryMatcher::Substring(_)) {
        return entries
            .iter()
            .filter(|e| type_filter.matches(e) && query.matches(&e.display_name()))
            .map(|e| (3u8, e.clone()))
            .collect();
    }

    let q = match &query.matcher {
        QueryMatcher::Substring(s) => s.clone(),
        _ => unreachable!(),
    };

    let mut scored: Vec<(u8, FileEntry)> = entries
        .iter()
        .filter(|e| type_filter.matches(e) && query.matches(&e.display_name()))
        .map(|e| (relevance_score(e, &q), e.clone()))
        .collect();

    scored.sort_by(|(sa, a), (sb, b)| {
        sa.cmp(sb).then_with(|| {
            let a_is_dir = matches!(a.kind, EntryKind::Directory);
            let b_is_dir = matches!(b.kind, EntryKind::Directory);
            a_is_dir.cmp(&b_is_dir)
        })
    });

    scored
}

/// Returns entries that match `query` and `type_filter`, sorted by relevance.
///
/// - Exact stem matches first
/// - Prefix matches second
/// - Word-boundary matches third
/// - Loose substring matches last
/// - Files before directories within the same tier
pub fn search(
    entries: &[FileEntry],
    query: &SearchQuery,
    type_filter: FileTypeFilter,
) -> Vec<FileEntry> {
    // For non-substring queries (glob, ext:) we skip relevance scoring and just filter.
    if !matches!(query.matcher, QueryMatcher::Substring(_)) {
        return entries
            .iter()
            .filter(|e| type_filter.matches(e) && query.matches(&e.display_name()))
            .cloned()
            .collect();
    }

    // Substring path: score + sort by relevance.
    let q = match &query.matcher {
        QueryMatcher::Substring(s) => s.clone(),
        _ => unreachable!(),
    };

    let mut scored: Vec<(u8, &FileEntry)> = entries
        .iter()
        .filter(|e| type_filter.matches(e) && query.matches(&e.display_name()))
        .map(|e| (relevance_score(e, &q), e))
        .collect();

    // Sort: primary = score (ascending), secondary = files before dirs.
    scored.sort_by(|(sa, a), (sb, b)| {
        sa.cmp(sb).then_with(|| {
            let a_is_dir = matches!(a.kind, EntryKind::Directory);
            let b_is_dir = matches!(b.kind, EntryKind::Directory);
            a_is_dir.cmp(&b_is_dir) // false < true → files first
        })
    });

    scored.into_iter().map(|(_, e)| e.clone()).collect()
}

/// Computes a relevance score for a substring query against a `FileEntry`.
///
/// Lower score = better match. Scores files against their **stem** (filename without
/// extension) so that `id.txt` scores 0 for query "id". When the query *includes* a dot
/// (e.g. "id.txt"), we also match against the full filename so it still scores 0.
fn relevance_score(entry: &FileEntry, query: &str) -> u8 {
    // Use the file stem (no extension) as the primary match target.
    let stem = entry
        .path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Full filename including extension (e.g. "id.txt").
    let full_name = entry
        .path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    let q = query; // already lowercase from QueryMatcher::Substring

    // Tier 0: stem exactly equals query ("id" == "id")
    //      OR full filename exactly equals query ("id.txt" == "id.txt")
    if stem == q || full_name == q {
        return 0;
    }

    // Tier 1: stem starts with the query ("identity", "id_card")
    //      OR full filename starts with the query ("id.txt" for query "id.")
    if stem.starts_with(q) || full_name.starts_with(q) {
        return 1;
    }

    // Tier 2: query appears at a word boundary in the stem.
    // Word separators: underscore, hyphen, space, dot.
    let words: Vec<&str> = stem.split(|c: char| !c.is_alphanumeric()).collect();
    if words.iter().any(|w| *w == q) {
        return 2;
    }

    // Tier 3: loose substring anywhere ("video" contains "id", "audio" contains "id")
    3
}

#[cfg(test)]
mod tests {
    use super::*;
    use bouma_core::entry::EntryKind;
    use std::ffi::OsString;
    use std::path::PathBuf;

    fn make_file(name: &str) -> FileEntry {
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

    fn make_dir(name: &str) -> FileEntry {
        FileEntry {
            name: OsString::from(name),
            path: PathBuf::from(format!("C:\\{name}")),
            kind: EntryKind::Directory,
            size: 0,
            created: None,
            modified: None,
            readonly: false,
            hidden: false,
        }
    }

    #[test]
    fn test_exact_stem_ranked_first() {
        let entries = vec![
            make_file("valid.mp4"),   // loose: "id" at end of "val-id"
            make_file("liquid.txt"),  // loose: "id" at end of "liqu-id"
            make_file("id.txt"),
            make_file("identity.pdf"),
        ];
        let query = SearchQuery::parse("id").unwrap();
        let results = search(&entries, &query, FileTypeFilter::All);

        // id.txt must come first (exact stem match)
        assert_eq!(results[0].display_name(), "id.txt");
        // identity.pdf second (prefix match)
        assert_eq!(results[1].display_name(), "identity.pdf");
        // valid and liquid last (loose substring)
        let loose: Vec<_> = results[2..].iter().map(|e| e.display_name()).collect();
        assert!(loose.contains(&"valid.mp4".to_string()));
        assert!(loose.contains(&"liquid.txt".to_string()));
    }


    #[test]
    fn test_word_boundary_ranked_above_loose() {
        let entries = vec![
            make_file("video.mp4"),   // loose: "id" in "video"
            make_file("user_id.txt"), // word boundary: "id" is a full word
        ];
        let query = SearchQuery::parse("id").unwrap();
        let results = search(&entries, &query, FileTypeFilter::All);

        assert_eq!(results[0].display_name(), "user_id.txt");
        assert_eq!(results[1].display_name(), "video.mp4");
    }

    #[test]
    fn test_files_ranked_before_dirs_same_tier() {
        let entries = vec![
            make_dir("identity"),  // dir, prefix match
            make_file("identity.pdf"), // file, prefix match
        ];
        let query = SearchQuery::parse("id").unwrap();
        let results = search(&entries, &query, FileTypeFilter::All);

        // Same tier (prefix), but file should come before dir
        assert_eq!(results[0].display_name(), "identity.pdf");
        assert_eq!(results[1].display_name(), "identity");
    }

    #[test]
    fn test_search_substring() {
        let entries = vec![
            make_file("report.pdf"),
            make_file("notes.txt"),
            make_file("final_report.pdf"),
        ];
        let query = SearchQuery::parse("report").unwrap();
        let results = search(&entries, &query, FileTypeFilter::All);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_glob() {
        let entries = vec![
            make_file("doc.pdf"),
            make_file("photo.jpg"),
            make_file("invoice.pdf"),
        ];
        let query = SearchQuery::parse("*.pdf").unwrap();
        let results = search(&entries, &query, FileTypeFilter::All);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_no_results() {
        let entries = vec![make_file("file.txt")];
        let query = SearchQuery::parse("xyz").unwrap();
        let results = search(&entries, &query, FileTypeFilter::All);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_preserves_order_for_glob() {
        let entries = vec![
            make_file("c_file.rs"),
            make_file("a_file.rs"),
            make_file("b_file.rs"),
        ];
        let query = SearchQuery::parse("ext:rs").unwrap();
        let results = search(&entries, &query, FileTypeFilter::All);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].display_name(), "c_file.rs");
        assert_eq!(results[1].display_name(), "a_file.rs");
        assert_eq!(results[2].display_name(), "b_file.rs");
    }
}
