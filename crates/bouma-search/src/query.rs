//! Search query parsing.
//!
//! Supports:
//! - Plain text: `project` → matches filenames containing "project"
//! - Glob patterns: `*.pdf` → matches by extension
//! - Extension filter: `ext:rs` → matches `.rs` files
//! - Future: `modified:this week`, `size:>1MB`

use bouma_core::error::{BoumaError, BoumaResult};
use globset::{Glob, GlobMatcher};

/// A parsed search query.
#[derive(Debug)]
pub struct SearchQuery {
    /// The raw query string as entered by the user.
    pub raw: String,

    /// Matching strategy derived from the query.
    pub matcher: QueryMatcher,
}

/// How to match entries against the query.
#[derive(Debug)]
pub enum QueryMatcher {
    /// Simple substring match (case-insensitive).
    Substring(String),

    /// Glob pattern match (e.g., `*.pdf`, `report_*`).
    Glob(GlobMatcher),

    /// Extension filter (e.g., `ext:rs`).
    Extension(String),
}

impl SearchQuery {
    /// Parses a raw query string into a `SearchQuery`.
    ///
    /// # Rules
    /// - `ext:xyz` → Extension filter
    /// - Contains `*` or `?` → Glob pattern
    /// - Otherwise → Substring match
    pub fn parse(raw: &str) -> BoumaResult<Self> {
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            return Err(BoumaError::InvalidQuery("Empty query".to_string()));
        }

        // Extension filter: "ext:rs"
        if let Some(ext) = trimmed.strip_prefix("ext:") {
            let ext = ext.trim().to_lowercase();
            if ext.is_empty() {
                return Err(BoumaError::InvalidQuery(
                    "Extension filter requires a value (e.g., ext:pdf)".to_string(),
                ));
            }
            return Ok(SearchQuery {
                raw: trimmed.to_string(),
                matcher: QueryMatcher::Extension(ext),
            });
        }

        // Glob pattern: contains * or ?
        if trimmed.contains('*') || trimmed.contains('?') {
            let glob = Glob::new(trimmed)
                .map_err(|e| BoumaError::InvalidQuery(format!("Invalid glob pattern: {e}")))?;
            return Ok(SearchQuery {
                raw: trimmed.to_string(),
                matcher: QueryMatcher::Glob(glob.compile_matcher()),
            });
        }

        // Default: substring match
        Ok(SearchQuery {
            raw: trimmed.to_string(),
            matcher: QueryMatcher::Substring(trimmed.to_lowercase()),
        })
    }

    /// Checks if the given filename matches this query.
    pub fn matches(&self, filename: &str) -> bool {
        match &self.matcher {
            QueryMatcher::Substring(pattern) => filename.to_lowercase().contains(pattern),
            QueryMatcher::Glob(matcher) => matcher.is_match(filename),
            QueryMatcher::Extension(ext) => {
                let lower = filename.to_lowercase();
                lower.ends_with(&format!(".{ext}"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substring_query() {
        let query = SearchQuery::parse("project").unwrap();
        assert!(query.matches("my_project.txt"));
        assert!(query.matches("PROJECT_NOTES.md"));
        assert!(!query.matches("readme.txt"));
    }

    #[test]
    fn test_glob_query() {
        let query = SearchQuery::parse("*.pdf").unwrap();
        assert!(query.matches("report.pdf"));
        assert!(query.matches("invoice.pdf"));
        assert!(!query.matches("report.txt"));
    }

    #[test]
    fn test_extension_filter() {
        let query = SearchQuery::parse("ext:rs").unwrap();
        assert!(query.matches("main.rs"));
        assert!(query.matches("lib.RS")); // case-insensitive
        assert!(!query.matches("main.py"));
    }

    #[test]
    fn test_empty_query_error() {
        let result = SearchQuery::parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_extension_error() {
        let result = SearchQuery::parse("ext:");
        assert!(result.is_err());
    }

    #[test]
    fn test_glob_with_question_mark() {
        let query = SearchQuery::parse("file?.txt").unwrap();
        assert!(query.matches("file1.txt"));
        assert!(query.matches("fileA.txt"));
        assert!(!query.matches("file12.txt"));
    }
}
