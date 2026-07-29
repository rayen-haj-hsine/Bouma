//! All application messages (the "Msg" in Elm architecture).
//!
//! Every user action and async result is represented as a `Message`.
//! The `update` function in `app.rs` pattern-matches on these to
//! determine state transitions.

use bouma_core::entry::{FileEntry, FileTypeFilter};
use bouma_core::operations::{OperationDiagnostics, OperationProgress};
use bouma_core::sort::SortField;
use std::path::PathBuf;

/// Display mode for the application interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    /// Hero Mind Map landing interface.
    MindMap,
    /// Standard file list interface.
    ListView,
}

/// Statistics emitted when a recursive search scan completes.
#[derive(Debug, Clone)]
pub struct SearchStats {
    /// The text that was searched.
    pub query: String,
    /// Total files/folders scanned during recursive walk.
    pub total_scanned: usize,
    /// How many results were found per tier (index = tier 0..=3).
    pub tier_counts: [usize; 4],
    /// How long the full scan took in milliseconds.
    pub scan_ms: u64,
    /// Max depth used for this scan.
    pub depth_used: usize,
}

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

    /// Directory contents loaded successfully with timing diagnostics.
    DirectoryLoaded(PathBuf, Vec<FileEntry>, OperationDiagnostics),

    /// Recursive search results loaded.
    /// Carries (tier, Vec<FileEntry>) groups (tier 0 = exact, 3 = partial) + scan stats.
    SearchResultsLoaded(Vec<(u8, Vec<FileEntry>)>, SearchStats),

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

    /// Fired after a short debounce delay to actually run the recursive search.
    /// Carries a generation counter — stale fires (gen < current) are silently dropped.
    SearchDebounced(u64),

    /// Filter by a specific file type category.
    FilterTypeSelected(FileTypeFilter),

    // ── Mind Map & View Modes ────────────────────────────────────

    /// Toggle whether a directory node is closed (pruned from search).
    ToggleFolderClosed(PathBuf),

    /// Toggle or set active view mode.
    SetViewMode(ViewMode),

    // ── Sidebar ─────────────────────────────────────────────────

    /// A sidebar item was clicked (e.g., a drive or favorite).
    SidebarNavigate(PathBuf),

    // ── Settings ────────────────────────────────────────────────

    /// Toggle showing hidden files.
    ToggleHidden,

    // ── File Operations ─────────────────────────────────────────

    /// Prompt user to create a new folder in current directory.
    CreateDirectorySubmit(String),

    /// Rename the currently selected entry.
    RenameSubmit(usize, String),

    /// Delete the currently selected entry to Recycle Bin.
    DeleteSelected,

    /// Copy the currently selected entry to clipboard buffer.
    CopySelected,

    /// Paste copied entry to current directory.
    Paste,

    /// Real-time progress update from background file operation.
    OperationProgressUpdate(OperationProgress),

    /// Background file operation completed.
    OperationFinished(Result<(), String>),
}
