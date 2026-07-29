//! Main application state and logic (the Elm "Model" + "Update").

use crate::message::Message;
use crate::theme;
use crate::views;

use bouma_cache::{HistoryStore, Settings};
use bouma_core::entry::{EntryKind, FileEntry};
use bouma_core::operations::{OperationDiagnostics, OperationProgress};
use bouma_core::sort::{sort_entries, SortField, SortOrder};
use bouma_search::SearchQuery;
use iced::widget::{column, container, row};
use iced::{Element, Length, Task, Theme};
use std::path::PathBuf;
use std::time::Instant;

/// The root application state.
pub struct Bouma {
    /// Current directory being displayed.
    current_path: PathBuf,

    /// All entries in the current directory (unsorted/unfiltered source of truth).
    all_entries: Vec<FileEntry>,

    /// Entries after sorting and optional search filtering.
    display_entries: Vec<FileEntry>,

    /// Currently selected entry index (into `display_entries`).
    selected_index: Option<usize>,

    /// Whether directory contents are being loaded.
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

    /// Active search query (None = show all).
    active_search: Option<SearchQuery>,

    /// Active file operation progress (for Transparency Panel).
    current_operation: Option<OperationProgress>,

    /// Diagnostic info timing breakdown (for Transparency Panel).
    current_diagnostics: Option<OperationDiagnostics>,

    /// Path copied to internal buffer (for Copy / Paste).
    clipboard_copy: Option<PathBuf>,
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
            active_search: None,
            current_operation: None,
            current_diagnostics: None,
            clipboard_copy: None,
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
            Message::OpenDirectory(path) => self.navigate_to(path),

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
                self.search_text.clear();
                self.active_search = None;
                self.refresh_display();
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

            // ── Search ──────────────────────────────────────────
            Message::SearchInputChanged(text) => {
                self.search_text = text;
                if self.search_text.is_empty() {
                    self.active_search = None;
                } else if let Ok(query) = SearchQuery::parse(&self.search_text) {
                    self.active_search = Some(query);
                }
                self.refresh_display();
                Task::none()
            }

            Message::SearchSubmit => {
                if !self.search_text.is_empty() {
                    if let Ok(query) = SearchQuery::parse(&self.search_text) {
                        self.active_search = Some(query);
                        self.refresh_display();
                    }
                }
                Task::none()
            }

            Message::SearchClear => {
                self.search_text.clear();
                self.active_search = None;
                self.refresh_display();
                Task::none()
            }

            // ── Sidebar ─────────────────────────────────────────
            Message::SidebarNavigate(path) => self.navigate_to(path),

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
                // Reload current directory to show changes
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
        );

        let sidebar = views::sidebar::view(&self.settings.favorites, &self.current_path);

        let file_list = views::file_list::view(
            &self.display_entries,
            self.selected_index,
            self.sort_field,
            self.sort_order,
            self.settings.show_hidden,
            self.is_loading,
        );

        let transparency = views::transparency_panel::view(
            self.current_operation.as_ref(),
            self.current_diagnostics.as_ref(),
        );

        let status_bar = views::status_bar::view(
            &self.display_entries,
            self.selected_index,
            self.settings.show_hidden,
        );

        let main_content = column![
            toolbar,
            row![sidebar, file_list],
            transparency,
            status_bar
        ]
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

    /// Refreshes `display_entries` from `all_entries` with current sort + search.
    fn refresh_display(&mut self) {
        let mut entries = if let Some(ref query) = self.active_search {
            bouma_search::search(&self.all_entries, query)
        } else {
            self.all_entries.clone()
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
