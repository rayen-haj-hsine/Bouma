//! Status bar view — shows item count, selection info, and total size.

use crate::message::Message;
use crate::theme;
use bouma_core::entry::{format_size, EntryKind, FileEntry};
use iced::widget::{container, row, text};
use iced::{Element, Length};

/// Renders the bottom status bar.
pub fn view<'a>(
    entries: &[FileEntry],
    selected_index: Option<usize>,
    show_hidden: bool,
) -> Element<'a, Message> {
    let visible: Vec<&FileEntry> = entries
        .iter()
        .filter(|e| show_hidden || !e.hidden)
        .collect();

    let dir_count = visible.iter().filter(|e| e.kind == EntryKind::Directory).count();
    let file_count = visible.len() - dir_count;

    let items_text = format!("{file_count} files, {dir_count} folders");

    let selection_text = if let Some(idx) = selected_index {
        if let Some(entry) = entries.get(idx) {
            if entry.kind == EntryKind::File {
                format!("  │  Selected: {} ({})", entry.display_name(), entry.display_size())
            } else {
                format!("  │  Selected: {}", entry.display_name())
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let total_size: u64 = visible
        .iter()
        .filter(|e| e.kind == EntryKind::File)
        .map(|e| e.size)
        .sum();

    let size_text = format!("  │  Total: {}", format_size(total_size));

    container(
        row![
            text(format!("{items_text}{selection_text}{size_text}"))
                .size(12)
                .color(theme::TEXT_SECONDARY),
        ]
        .align_y(iced::Alignment::Center),
    )
    .padding([0.0, theme::PADDING])
    .height(theme::STATUS_BAR_HEIGHT)
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
