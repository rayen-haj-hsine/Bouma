//! Main application state and logic (the Elm "Model" + "Update").

use crate::message::{Message, SearchStats, ViewMode};
use crate::theme;
use crate::views;

use bouma_cache::{HistoryStore, Settings};
use bouma_core::entry::{EntryKind, FileEntry, FileTypeFilter};
use bouma_core::operations::{OperationDiagnostics, OperationProgress};
use bouma_core::sort::{sort_entries, SortField, SortOrder};
use bouma_search::SearchQuery;
use iced::widget::{column, container, row};
use iced::{Element, Length, Task, Theme};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// The root application state.
pub struct Bouma {
    /// Current directory being displayed.
    current_path: PathBuf,

    /// All entries in the current directory (unsorted/unfiltered source of truth).
    all_entries: Vec<FileEntry>,

    /// Entries after sorting, type filtering, and search filtering.
    display_entries: Vec<FileEntry>,

    /// Currently selected entry index (into `display_entries`).
    selected_index: Option<usize>,

    /// Whether directory contents or search results are being loaded.
    is_loading: bool,

    /// Current error message (if any).
    error: Option<String>,

    /// Navigation history (back/forward).
    history: HistoryStore,

    /// Application settings.
    settings: Settings,

    /// Current sort field.
    sort_field: SortField,

    /// Current sort order.
    sort_order: SortOrder,

    /// Search bar text.
    search_text: String,

    /// Monotonically increasing counter — incremented on every keystroke.
    /// `SearchDebounced(gen)` messages with a stale `gen` are silently dropped.
    search_generation: u64,

    /// Selected file type filter (All, Folders, Documents, Images, etc.).
    type_filter: FileTypeFilter,

    /// Active search query (None = show all).
    active_search: Option<SearchQuery>,

    /// Active file operation progress (for Transparency Panel).
    current_operation: Option<OperationProgress>,

    /// Diagnostic info timing breakdown (for Transparency Panel).
    current_diagnostics: Option<OperationDiagnostics>,

    /// Path copied to internal buffer (for Copy / Paste).
    clipboard_copy: Option<PathBuf>,

    /// Grouped search results (tier 0..=3) — Some only when a search is active.
    search_groups: Option<Vec<(u8, Vec<FileEntry>)>>,

    /// Stats from the last completed recursive scan — shown in the transparency panel.
    search_stats: Option<SearchStats>,

    /// Active view mode (MindMap vs ListView).
    view_mode: ViewMode,

    /// Folders toggled as "Closed" (pruned from recursive search).
    closed_folders: HashSet<PathBuf>,
}

impl Bouma {
    /// Creates the application with initial state and a startup command.
    pub fn new() -> (Self, Task<Message>) {
        let settings = Settings::load();
        let start_dir = settings.start_directory.clone();

        let app = Bouma {
            current_path: start_dir.clone(),
            all_entries: Vec::new(),
            display_entries: Vec::new(),
            selected_index: None,
            is_loading: true,
            error: None,
            history: HistoryStore::new(),
            sort_field: settings.sort_field,
            sort_order: settings.sort_order,
            search_text: String::new(),
            search_generation: 0,
            type_filter: FileTypeFilter::All,
            active_search: None,
            current_operation: None,
            current_diagnostics: None,
            clipboard_copy: None,
            search_groups: None,
            search_stats: None,
            view_mode: ViewMode::MindMap,
            closed_folders: HashSet::new(),
            settings,
        };

        // Load the starting directory on launch
        let task = Task::perform(load_directory(start_dir.clone()), move |result| match result {
            Ok((entries, diag)) => Message::DirectoryLoaded(start_dir.clone(), entries, diag),
            Err(err) => Message::DirectoryError(err),
        });

        (app, task)
    }

    /// Returns the window title.
    pub fn title(&self) -> String {
        format!("Bouma — {}", self.current_path.display())
    }

    /// Returns the application theme (dark).
    pub fn theme(&self) -> Theme {
        Theme::Dark
    }

    /// Handles all messages (the "update" function in Elm architecture).
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // ── Navigation ──────────────────────────────────────
            Message::OpenDirectory(path) => {
                self.view_mode = ViewMode::ListView;
                self.navigate_to(path)
            }

            Message::GoUp => {
                if let Some(parent) = self.current_path.parent() {
                    let parent = parent.to_path_buf();
                    self.navigate_to(parent)
                } else {
                    Task::none()
                }
            }

            Message::GoBack => {
                let current = self.current_path.clone();
                if let Some(prev) = self.history.go_back(&current) {
                    self.load_directory_async(prev)
                } else {
                    Task::none()
                }
            }

            Message::GoForward => {
                let current = self.current_path.clone();
                if let Some(next) = self.history.go_forward(&current) {
                    self.load_directory_async(next)
                } else {
                    Task::none()
                }
            }

            // ── Directory loading ───────────────────────────────
            Message::DirectoryLoaded(path, entries, diag) => {
                self.current_path = path;
                self.all_entries = entries;
                self.current_diagnostics = Some(diag);
                self.is_loading = false;
                self.error = None;
                self.selected_index = None;

                if !self.search_text.is_empty() {
                    if let Ok(query) = SearchQuery::parse(&self.search_text) {
                        self.active_search = Some(query);
                        return self.trigger_recursive_search();
                    }
                }

                self.refresh_display();
                Task::none()
            }

            Message::SearchResultsLoaded(groups, stats) => {
                self.is_loading = false;
                self.display_entries = groups
                    .iter()
                    .flat_map(|(_, entries)| entries.iter().cloned())
                    .collect();
                self.search_groups = Some(groups);
                self.search_stats = Some(stats);
                self.selected_index = None;
                Task::none()
            }

            Message::DirectoryError(err) => {
                self.is_loading = false;
                self.error = Some(err);
                tracing::error!("Directory error: {:?}", self.error);
                Task::none()
            }

            // ── File interactions ───────────────────────────────
            Message::EntryClicked(index) => {
                if self.selected_index == Some(index) {
                    return self.update(Message::EntryDoubleClicked(index));
                }
                self.selected_index = Some(index);
                Task::none()
            }

            Message::EntryDoubleClicked(index) => {
                if let Some(entry) = self.display_entries.get(index) {
                    match entry.kind {
                        EntryKind::Directory => {
                            let path = entry.path.clone();
                            self.navigate_to(path)
                        }
                        EntryKind::File | EntryKind::Symlink => {
                            let path = entry.path.clone();
                            let _ = open::that(&path);
                            Task::none()
                        }
                    }
                } else {
                    Task::none()
                }
            }

            // ── Sorting ─────────────────────────────────────────
            Message::ToggleSort(field) => {
                if self.sort_field == field {
                    self.sort_order = self.sort_order.toggle();
                } else {
                    self.sort_field = field;
                    self.sort_order = SortOrder::Ascending;
                }
                self.refresh_display();
                Task::none()
            }

            // ── Search & Filter ─────────────────────────────────
            Message::SearchInputChanged(text) => {
                self.search_text = text;
                if self.search_text.is_empty() {
                    self.active_search = None;
                    self.search_groups = None;
                    self.search_stats = None;
                    self.refresh_display();
                    Task::none()
                } else if let Ok(query) = SearchQuery::parse(&self.search_text) {
                    self.active_search = Some(query);
                    self.view_mode = ViewMode::ListView; // Switch to list view on search
                    self.refresh_display();

                    let path_depth = self.current_path.components().count();
                    let delay_ms: u64 = if path_depth <= 2 { 350 } else { 200 };
                    self.search_generation += 1;
                    let gen = self.search_generation;
                    Task::perform(
                        async move {
                            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                            gen
                        },
                        Message::SearchDebounced,
                    )
                } else {
                    self.refresh_display();
                    Task::none()
                }
            }

            Message::SearchDebounced(gen) => {
                if gen == self.search_generation {
                    return self.trigger_recursive_search();
                }
                Task::none()
            }

            Message::SearchSubmit => {
                if !self.search_text.is_empty() {
                    if let Ok(query) = SearchQuery::parse(&self.search_text) {
                        self.active_search = Some(query);
                        self.view_mode = ViewMode::ListView;
                        return self.trigger_recursive_search();
                    }
                }
                Task::none()
            }

            Message::SearchClear => {
                self.search_text.clear();
                self.active_search = None;
                self.search_groups = None;
                self.search_stats = None;
                self.refresh_display();
                Task::none()
            }

            Message::FilterTypeSelected(filter) => {
                self.type_filter = filter;
                if !self.search_text.is_empty() {
                    self.trigger_recursive_search()
                } else {
                    self.refresh_display();
                    Task::none()
                }
            }

            // ── Mind Map & View Modes ────────────────────────────
            Message::SetViewMode(mode) => {
                self.view_mode = mode;
                Task::none()
            }

            Message::ToggleFolderClosed(path) => {
                if self.closed_folders.contains(&path) {
                    self.closed_folders.remove(&path);
                } else {
                    self.closed_folders.insert(path);
                }
                if self.active_search.is_some() {
                    return self.trigger_recursive_search();
                }
                Task::none()
            }

            // ── Sidebar ─────────────────────────────────────────
            Message::SidebarNavigate(path) => {
                self.view_mode = ViewMode::ListView;
                self.navigate_to(path)
            }

            // ── Settings ────────────────────────────────────────
            Message::ToggleHidden => {
                self.settings.show_hidden = !self.settings.show_hidden;
                self.refresh_display();
                Task::none()
            }

            // ── File Operations ─────────────────────────────────
            Message::CreateDirectorySubmit(folder_name) => {
                let parent = self.current_path.clone();
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            bouma_filesystem::create_directory(&parent, &folder_name)
                        })
                        .await
                        .map_err(|e| e.to_string())?
                        .map_err(|e| e.to_string())
                    },
                    |res| match res {
                        Ok(_) => Message::OperationFinished(Ok(())),
                        Err(e) => Message::OperationFinished(Err(e)),
                    },
                )
            }

            Message::RenameSubmit(index, new_name) => {
                if let Some(entry) = self.display_entries.get(index) {
                    let from = entry.path.clone();
                    Task::perform(
                        async move {
                            tokio::task::spawn_blocking(move || {
                                bouma_filesystem::rename_entry(&from, &new_name)
                            })
                            .await
                            .map_err(|e| e.to_string())?
                            .map_err(|e| e.to_string())
                        },
                        |res| match res {
                            Ok(_) => Message::OperationFinished(Ok(())),
                            Err(e) => Message::OperationFinished(Err(e)),
                        },
                    )
                } else {
                    Task::none()
                }
            }

            Message::DeleteSelected => {
                if let Some(idx) = self.selected_index {
                    if let Some(entry) = self.display_entries.get(idx) {
                        let path = entry.path.clone();
                        return Task::perform(
                            async move {
                                tokio::task::spawn_blocking(move || {
                                    bouma_filesystem::delete_entry(&path)
                                })
                                .await
                                .map_err(|e| e.to_string())?
                                .map_err(|e| e.to_string())
                            },
                            |res| match res {
                                Ok(_) => Message::OperationFinished(Ok(())),
                                Err(e) => Message::OperationFinished(Err(e)),
                            },
                        );
                    }
                }
                Task::none()
            }

            Message::CopySelected => {
                if let Some(idx) = self.selected_index {
                    if let Some(entry) = self.display_entries.get(idx) {
                        self.clipboard_copy = Some(entry.path.clone());
                    }
                }
                Task::none()
            }

            Message::Paste => {
                if let Some(ref src) = self.clipboard_copy {
                    let src = src.clone();
                    let dst_dir = self.current_path.clone();

                    return Task::perform(
                        async move {
                            tokio::task::spawn_blocking(move || {
                                bouma_filesystem::copy_entry(&src, &dst_dir, None)
                            })
                            .await
                            .map_err(|e| e.to_string())?
                            .map_err(|e| e.to_string())
                        },
                        |res| match res {
                            Ok(_) => Message::OperationFinished(Ok(())),
                            Err(e) => Message::OperationFinished(Err(e)),
                        },
                    );
                }
                Task::none()
            }

            Message::OperationProgressUpdate(progress) => {
                self.current_operation = Some(progress);
                Task::none()
            }

            Message::OperationFinished(result) => {
                self.current_operation = None;
                if let Err(e) = result {
                    self.error = Some(e);
                }
                self.load_directory_async(self.current_path.clone())
            }
        }
    }

    /// Composes the full UI view.
    pub fn view(&self) -> Element<'_, Message> {
        let toolbar = views::toolbar::view(
            &self.current_path.to_string_lossy(),
            self.history.can_go_back(),
            self.history.can_go_forward(),
            &self.search_text,
            self.type_filter,
            self.view_mode,
        );

        let main_view: Element<'_, Message> = if self.view_mode == ViewMode::MindMap && self.active_search.is_none() {
            views::mind_map::view(
                &self.current_path,
                &self.closed_folders,
                &self.search_text,
                self.type_filter,
            )
        } else {
            let sidebar = views::sidebar::view(&self.settings.favorites, &self.current_path);

            let file_list = views::file_list::view(
                &self.display_entries,
                self.search_groups.as_deref(),
                self.selected_index,
                self.sort_field,
                self.sort_order,
                self.settings.show_hidden,
                self.is_loading,
                &self.current_path,
            );

            row![sidebar, file_list].into()
        };

        let transparency = views::transparency_panel::view(
            self.current_operation.as_ref(),
            self.current_diagnostics.as_ref(),
            self.search_stats.as_ref(),
        );

        let status_bar = views::status_bar::view(
            &self.display_entries,
            self.selected_index,
            self.settings.show_hidden,
        );

        let main_content = column![toolbar, main_view, transparency, status_bar]
            .width(Length::Fill)
            .height(Length::Fill);

        container(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(theme::BG_BASE)),
                ..Default::default()
            })
            .into()
    }

    // ── Internal helpers ────────────────────────────────────────

    /// Navigates to a new directory (updates history and loads).
    fn navigate_to(&mut self, path: PathBuf) -> Task<Message> {
        self.history.navigate(&self.current_path, path.clone());
        self.load_directory_async(path)
    }

    /// Starts async directory loading with transparency diagnostic tracking.
    fn load_directory_async(&mut self, path: PathBuf) -> Task<Message> {
        self.is_loading = true;
        self.error = None;
        self.selected_index = None;

        let path_clone = path.clone();
        Task::perform(load_directory(path), move |result| match result {
            Ok((entries, diag)) => Message::DirectoryLoaded(path_clone.clone(), entries, diag),
            Err(err) => Message::DirectoryError(err),
        })
    }

    /// Triggers async background recursive search on subfolders with pruning of closed paths.
    fn trigger_recursive_search(&mut self) -> Task<Message> {
        let root = self.current_path.clone();
        let query_text = self.search_text.clone();
        let type_filter = self.type_filter;
        let closed_folders = self.closed_folders.clone();

        // Adaptive depth based on filesystem depth.
        let path_depth = root.components().count();
        let max_depth: usize = match path_depth {
            0 | 1 => 8,
            2 | 3 => 7,
            4 | 5 => 6,
            _ => 5,
        };

        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    let query = SearchQuery::parse(&query_text).ok()?;
                    let t0 = Instant::now();

                    // Parallel recursive scan via jwalk with closed folder pruning
                    let all_entries = bouma_filesystem::walk_directory_pruned(
                        &root,
                        max_depth,
                        &closed_folders,
                    )
                    .ok()?;
                    let total_scanned = all_entries.len();

                    // Score every matching result
                    let scored = bouma_search::search_scored(&all_entries, &query, type_filter);

                    // Cap to 2000 total results on wide scans
                    let scored: Vec<_> = scored.into_iter().take(2000).collect();

                    // Aggregate tier counts before grouping
                    let mut tier_counts = [0usize; 4];
                    for (tier, _) in &scored {
                        tier_counts[*tier as usize] += 1;
                    }

                    // Group by tier: Vec<(tier, Vec<FileEntry>)>
                    let mut groups: Vec<(u8, Vec<FileEntry>)> = Vec::new();
                    for tier in 0u8..=3u8 {
                        let tier_entries: Vec<FileEntry> = scored
                            .iter()
                            .filter(|(t, _)| *t == tier)
                            .map(|(_, e)| e.clone())
                            .collect();
                        if !tier_entries.is_empty() {
                            groups.push((tier, tier_entries));
                        }
                    }

                    let stats = SearchStats {
                        query: query_text.clone(),
                        total_scanned,
                        tier_counts,
                        scan_ms: t0.elapsed().as_millis() as u64,
                        depth_used: max_depth,
                    };

                    Some((groups, stats))
                })
                .await
                .ok()
                .flatten()
            },
            |result| match result {
                Some((groups, stats)) => Message::SearchResultsLoaded(groups, stats),
                None => Message::SearchResultsLoaded(
                    vec![],
                    SearchStats {
                        query: String::new(),
                        total_scanned: 0,
                        tier_counts: [0; 4],
                        scan_ms: 0,
                        depth_used: 0,
                    },
                ),
            },
        )
    }

    /// Refreshes `display_entries` from `all_entries` with current sort + search + type filter.
    fn refresh_display(&mut self) {
        let mut entries = if let Some(ref query) = self.active_search {
            bouma_search::search(&self.all_entries, query, self.type_filter)
        } else {
            self.all_entries
                .iter()
                .filter(|e| self.type_filter.matches(e))
                .cloned()
                .collect()
        };

        sort_entries(&mut entries, self.sort_field, self.sort_order);
        self.display_entries = entries;
        self.selected_index = None;
    }
}

/// Loads a directory's contents with timing diagnostics.
async fn load_directory(
    path: PathBuf,
) -> Result<(Vec<FileEntry>, OperationDiagnostics), String> {
    tokio::task::spawn_blocking(move || {
        let mut diag = OperationDiagnostics::new("Folder Loading");
        let start = Instant::now();

        let entries = bouma_filesystem::read_directory(&path).map_err(|e| e.to_string())?;
        diag.record_phase("Read Filesystem", start.elapsed());

        Ok((entries, diag))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}
