//! File list view — the main content area showing directory entries.

use crate::message::Message;
use crate::theme;
use bouma_core::entry::{EntryKind, FileEntry};
use bouma_core::sort::{SortField, SortOrder};
use iced::widget::{button, column, container, row, scrollable, text, Column};
use iced::{Element, Length};

/// Renders the file list panel.
pub fn view<'a>(
    entries: &'a [FileEntry],
    selected_index: Option<usize>,
    sort_field: SortField,
    sort_order: SortOrder,
    show_hidden: bool,
    is_loading: bool,
) -> Element<'a, Message> {
    let mut content = Column::new().spacing(0);

    // ── Column headers ──────────────────────────────────────────
    content = content.push(column_headers(sort_field, sort_order));

    // ── Loading state ───────────────────────────────────────────
    if is_loading {
        return container(
            column![
                column_headers(sort_field, sort_order),
                container(text("Loading...").size(14).color(theme::TEXT_SECONDARY))
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
            ]
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(theme::BG_BASE)),
            ..Default::default()
        })
        .into();
    }

    // ── Empty state ─────────────────────────────────────────────
    let visible_entries: Vec<(usize, &FileEntry)> = entries
        .iter()
        .enumerate()
        .filter(|(_, e)| show_hidden || !e.hidden)
        .collect();

    if visible_entries.is_empty() {
        return container(
            column![
                column_headers(sort_field, sort_order),
                container(
                    text("This folder is empty")
                        .size(14)
                        .color(theme::TEXT_MUTED)
                )
                .center_x(Length::Fill)
                .center_y(Length::Fill)
            ]
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(theme::BG_BASE)),
            ..Default::default()
        })
        .into();
    }

    // ── File rows ───────────────────────────────────────────────
    let mut rows = Column::new().spacing(0);

    for (idx, entry) in visible_entries {
        let is_selected = selected_index == Some(idx);
        rows = rows.push(file_row(entry, idx, is_selected));
    }

    container(
        column![
            content,
            scrollable(rows.width(Length::Fill)).height(Length::Fill)
        ]
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_theme| container::Style {
        background: Some(iced::Background::Color(theme::BG_BASE)),
        ..Default::default()
    })
    .into()
}

/// Renders the column header row (Name, Size, Modified) with sort indicators.
fn column_headers(sort_field: SortField, sort_order: SortOrder) -> Element<'static, Message> {
    let arrow = match sort_order {
        SortOrder::Ascending => " ▲",
        SortOrder::Descending => " ▼",
    };

    let name_label = format!("Name{}", if sort_field == SortField::Name { arrow } else { "" });
    let size_label = format!("Size{}", if sort_field == SortField::Size { arrow } else { "" });
    let modified_label = format!(
        "Modified{}",
        if sort_field == SortField::Modified { arrow } else { "" }
    );

    container(
        row![
            header_button(name_label, SortField::Name, Length::Fill),
            header_button(size_label, SortField::Size, Length::Fixed(100.0)),
            header_button(modified_label, SortField::Modified, Length::Fixed(160.0)),
        ]
        .spacing(0),
    )
    .padding([0.0, theme::PADDING])
    .height(theme::ROW_HEIGHT)
    .width(Length::Fill)
    .style(|_theme| container::Style {
        background: Some(iced::Background::Color(theme::BG_SURFACE)),
        border: iced::Border {
            color: theme::BORDER,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}

/// A clickable column header.
fn header_button<'a>(label: String, field: SortField, width: Length) -> Element<'a, Message> {
    button(text(label).size(12).color(theme::TEXT_SECONDARY))
        .on_press(Message::ToggleSort(field))
        .width(width)
        .padding([6, 4])
        .style(|_theme, status| {
            let bg = match status {
                button::Status::Hovered => theme::BG_ELEVATED,
                _ => iced::Color::TRANSPARENT,
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: theme::TEXT_SECONDARY,
                border: iced::Border {
                    color: iced::Color::TRANSPARENT,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

/// Renders a single file/directory row.
fn file_row(entry: &FileEntry, index: usize, is_selected: bool) -> Element<'_, Message> {
    let icon = match entry.kind {
        EntryKind::Directory => "📁",
        EntryKind::File => file_icon(entry),
        EntryKind::Symlink => "🔗",
    };

    let name_color = if is_selected {
        theme::ACCENT
    } else {
        theme::TEXT_PRIMARY
    };

    let name_text = row![
        text(icon).size(14),
        text(entry.display_name()).size(13).color(name_color),
    ]
    .spacing(theme::SPACING_SM)
    .align_y(iced::Alignment::Center)
    .width(Length::Fill);

    let size_text = text(entry.display_size())
        .size(12)
        .color(theme::TEXT_SECONDARY)
        .width(100);

    let modified_text = text(format_time(entry.modified))
        .size(12)
        .color(theme::TEXT_SECONDARY)
        .width(160);

    let row_content = row![name_text, size_text, modified_text]
        .spacing(0)
        .align_y(iced::Alignment::Center);

    button(row_content)
        .on_press(Message::EntryClicked(index))
        .width(Length::Fill)
        .padding([0.0, theme::PADDING])
        .style(move |_theme, status| {
            let bg = if is_selected {
                theme::BG_SELECTED
            } else {
                match status {
                    button::Status::Hovered => theme::BG_ELEVATED,
                    _ => iced::Color::TRANSPARENT,
                }
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: theme::TEXT_PRIMARY,
                border: iced::Border {
                    color: iced::Color::TRANSPARENT,
                    width: 0.0,
                    radius: 2.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

/// Returns an emoji icon based on file extension.
fn file_icon(entry: &FileEntry) -> &'static str {
    match entry.extension().as_deref() {
        Some("pdf") => "📕",
        Some("doc" | "docx" | "odt") => "📄",
        Some("xls" | "xlsx" | "ods") => "📊",
        Some("ppt" | "pptx" | "odp") => "📽",
        Some("jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp") => "🖼",
        Some("mp3" | "wav" | "flac" | "ogg" | "aac") => "🎵",
        Some("mp4" | "mkv" | "avi" | "mov" | "webm") => "🎬",
        Some("zip" | "rar" | "7z" | "tar" | "gz") => "📦",
        Some("exe" | "msi") => "⚙",
        Some("rs" | "py" | "js" | "ts" | "c" | "cpp" | "h" | "java" | "go") => "💻",
        Some("md" | "txt" | "log") => "📝",
        Some("json" | "toml" | "yaml" | "yml" | "xml") => "📋",
        _ => "📄",
    }
}

/// Formats a `SystemTime` into a human-readable date string.
fn format_time(time: Option<std::time::SystemTime>) -> String {
    let Some(time) = time else {
        return "—".to_string();
    };

    let Ok(duration) = time.duration_since(std::time::UNIX_EPOCH) else {
        return "—".to_string();
    };

    // Simple date formatting without external crate
    let secs = duration.as_secs();
    let days = secs / 86400;
    let years_approx = 1970 + (days / 365);
    let day_in_year = days % 365;
    let month_approx = (day_in_year / 30) + 1;
    let day_approx = (day_in_year % 30) + 1;

    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let month_name = months
        .get((month_approx as usize).saturating_sub(1))
        .unwrap_or(&"???");

    format!("{month_name} {day_approx:02}, {years_approx}")
}
