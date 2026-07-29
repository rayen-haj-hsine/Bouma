//! File list view — flat directory listing or grouped search results.

use crate::message::Message;
use crate::theme;
use bouma_core::entry::{EntryKind, FileEntry};
use bouma_core::sort::{SortField, SortOrder};
use iced::widget::{button, column, container, row, scrollable, text, Column};
use iced::{Color, Element, Length};
use std::path::Path;

/// Tier labels and accent colours for grouped search results.
const TIER_META: [(u8, &str, Color); 4] = [
    (0, "Exact Match", Color { r: 0.35, g: 0.85, b: 0.60, a: 1.0 }),
    (1, "Starts With", Color { r: 0.45, g: 0.70, b: 1.00, a: 1.0 }),
    (2, "Word Match",  Color { r: 0.90, g: 0.75, b: 0.30, a: 1.0 }),
    (3, "Partial",     Color { r: 0.65, g: 0.65, b: 0.65, a: 1.0 }),
];

/// Renders the file list panel.
///
/// When `search_groups` is `Some`, renders grouped search results with section headers.
/// When `None`, renders a flat sorted directory listing.
pub fn view<'a>(
    entries: &'a [FileEntry],
    search_groups: Option<&'a [(u8, Vec<FileEntry>)]>,
    selected_index: Option<usize>,
    sort_field: SortField,
    sort_order: SortOrder,
    show_hidden: bool,
    is_loading: bool,
    search_root: &'a Path,
) -> Element<'a, Message> {
    // ── Loading state ────────────────────────────────────────────
    if is_loading {
        return container(
            column![
                column_headers(sort_field, sort_order),
                container(text("Searching…").size(14).color(theme::TEXT_SECONDARY))
                    .center_x(Length::Fill)
                    .center_y(Length::Fill),
            ]
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(bg_style())
        .into();
    }

    // ── Grouped search results ────────────────────────────────────
    if let Some(groups) = search_groups {
        return render_search_groups(groups, show_hidden, search_root);
    }

    // ── Flat directory listing ────────────────────────────────────
    let visible: Vec<(usize, &FileEntry)> = entries
        .iter()
        .enumerate()
        .filter(|(_, e)| show_hidden || !e.hidden)
        .collect();

    if visible.is_empty() {
        return container(
            column![
                column_headers(sort_field, sort_order),
                container(
                    text("This folder is empty")
                        .size(14)
                        .color(theme::TEXT_MUTED)
                )
                .center_x(Length::Fill)
                .center_y(Length::Fill),
            ]
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(bg_style())
        .into();
    }

    let mut rows = Column::new().spacing(0);
    for (idx, entry) in visible {
        let is_selected = selected_index == Some(idx);
        rows = rows.push(flat_row(entry, idx, is_selected));
    }

    container(
        column![
            column_headers(sort_field, sort_order),
            scrollable(rows.width(Length::Fill)).height(Length::Fill),
        ]
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(bg_style())
    .into()
}

// ── Grouped search rendering ─────────────────────────────────────────────────

fn render_search_groups<'a>(
    groups: &'a [(u8, Vec<FileEntry>)],
    show_hidden: bool,
    search_root: &'a Path,
) -> Element<'a, Message> {
    if groups.is_empty() {
        return container(
            container(
                text("No results found")
                    .size(14)
                    .color(theme::TEXT_MUTED),
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(bg_style())
        .into();
    }

    let mut content = Column::new().spacing(0);
    // Running index across all visible entries (for selection)
    let mut global_idx: usize = 0;

    for (tier, entries) in groups {
        let visible: Vec<&FileEntry> = entries
            .iter()
            .filter(|e| show_hidden || !e.hidden)
            .collect();
        if visible.is_empty() {
            global_idx += entries.len();
            continue;
        }

        let (_, label, accent) = TIER_META
            .iter()
            .find(|(t, _, _)| t == tier)
            .copied()
            .unwrap_or((3, "Partial", theme::TEXT_MUTED));

        // Section header
        content = content.push(section_header(label, visible.len(), accent));

        for entry in visible {
            let idx = global_idx;
            content = content.push(search_row(entry, idx, false, search_root));
            global_idx += 1;
        }
    }

    container(
        scrollable(content.width(Length::Fill)).height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(bg_style())
    .into()
}

/// A bold section header with a coloured left accent bar.
fn section_header<'a>(label: &'a str, count: usize, accent: Color) -> Element<'a, Message> {
    container(
        row![
            // Coloured left accent bar
            container(column![])
                .width(3)
                .height(Length::Fill)
                .style(move |_theme| container::Style {
                    background: Some(iced::Background::Color(accent)),
                    ..Default::default()
                }),
            text(format!("  {label}"))
                .size(11)
                .color(accent),
            text(format!("  ·  {count} result{}", if count == 1 { "" } else { "s" }))
                .size(10)
                .color(theme::TEXT_MUTED),
        ]
        .align_y(iced::Alignment::Center)
        .spacing(0),
    )
    .padding([4.0, theme::PADDING])
    .width(Length::Fill)
    .style(|_theme| container::Style {
        background: Some(iced::Background::Color(Color {
            r: 0.11,
            g: 0.11,
            b: 0.13,
            a: 1.0,
        })),
        border: iced::Border {
            color: Color { r: 0.20, g: 0.20, b: 0.24, a: 1.0 },
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}

/// A richer row for search results: icon + name + relative path + size + modified.
fn search_row<'a>(
    entry: &'a FileEntry,
    index: usize,
    is_selected: bool,
    search_root: &'a Path,
) -> Element<'a, Message> {
    let icon = match entry.kind {
        EntryKind::Directory => "📁",
        EntryKind::File => file_icon(entry),
        EntryKind::Symlink => "🔗",
    };

    let name_color = if is_selected { theme::ACCENT } else { theme::TEXT_PRIMARY };

    // Compute path relative to search root for context
    let rel_path = entry
        .path
        .parent()
        .and_then(|p| p.strip_prefix(search_root).ok())
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();

    let name_col = column![
        row![
            text(icon).size(13),
            text(entry.display_name()).size(13).color(name_color),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
        if !rel_path.is_empty() {
            Element::from(text(format!("  …/{rel_path}"))
                .size(10)
                .color(theme::TEXT_MUTED))
        } else {
            Element::from(text("  root")
                .size(10)
                .color(theme::TEXT_MUTED))
        },
    ]
    .spacing(1)
    .width(Length::Fill);

    let size_text = text(entry.display_size())
        .size(11)
        .color(theme::TEXT_SECONDARY)
        .width(90);

    let modified_text = text(format_time(entry.modified))
        .size(11)
        .color(theme::TEXT_SECONDARY)
        .width(140);

    let row_content = row![name_col, size_text, modified_text]
        .spacing(0)
        .align_y(iced::Alignment::Center);

    button(row_content)
        .on_press(Message::EntryClicked(index))
        .width(Length::Fill)
        .padding([4.0, theme::PADDING])
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

// ── Flat directory row ───────────────────────────────────────────────────────

fn flat_row(entry: &FileEntry, index: usize, is_selected: bool) -> Element<'_, Message> {
    let icon = match entry.kind {
        EntryKind::Directory => "📁",
        EntryKind::File => file_icon(entry),
        EntryKind::Symlink => "🔗",
    };

    let name_color = if is_selected { theme::ACCENT } else { theme::TEXT_PRIMARY };

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

// ── Shared helpers ───────────────────────────────────────────────────────────

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

fn format_time(time: Option<std::time::SystemTime>) -> String {
    let Some(time) = time else { return "—".to_string() };
    let Ok(duration) = time.duration_since(std::time::UNIX_EPOCH) else { return "—".to_string() };

    let secs = duration.as_secs();
    let days = secs / 86400;
    let years_approx = 1970 + (days / 365);
    let day_in_year = days % 365;
    let month_approx = (day_in_year / 30) + 1;
    let day_approx = (day_in_year % 30) + 1;

    let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let month_name = months.get((month_approx as usize).saturating_sub(1)).unwrap_or(&"???");
    format!("{month_name} {day_approx:02}, {years_approx}")
}

fn bg_style() -> impl Fn(&iced::Theme) -> container::Style {
    |_theme| container::Style {
        background: Some(iced::Background::Color(theme::BG_BASE)),
        ..Default::default()
    }
}
