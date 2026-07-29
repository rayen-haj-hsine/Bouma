//! All application messages (the "Msg" in Elm architecture).
//!
//! Every user action and async result is represented as a `Message`.
//! The `update` function in `app.rs` pattern-matches on these to
//! determine state transitions.

use bouma_core::entry::FileEntry;
use bouma_core::sort::SortField;
use std::path::PathBuf;

/// Application-level messages.
#[derive(Debug, Clone)]
pub enum Message {
    // ── Navigation ──────────────────────────────────────────────

    /// Navigate into a directory.
    OpenDirectory(PathBuf),

    /// Navigate to the parent directory.
    GoUp,

    /// Navigate back in history.
    GoBack,

    /// Navigate forward in history.
    GoForward,

    // ── Directory loading ───────────────────────────────────────

    /// Directory contents loaded successfully.
    DirectoryLoaded(PathBuf, Vec<FileEntry>),

    /// Directory loading failed.
    DirectoryError(String),

    // ── File interactions ───────────────────────────────────────

    /// A file entry was clicked (selected).
    EntryClicked(usize),

    /// A file entry was double-clicked (open).
    EntryDoubleClicked(usize),

    // ── Sorting ─────────────────────────────────────────────────

    /// Change the sort column. Clicking the same column toggles order.
    ToggleSort(SortField),

    // ── Search ──────────────────────────────────────────────────

    /// The search input text changed.
    SearchInputChanged(String),

    /// Execute the current search.
    SearchSubmit,

    /// Clear the search and show all entries.
    SearchClear,

    // ── Sidebar ─────────────────────────────────────────────────

    /// A sidebar item was clicked (e.g., a drive or favorite).
    SidebarNavigate(PathBuf),

    // ── Settings ────────────────────────────────────────────────

    /// Toggle showing hidden files.
    ToggleHidden,
}
