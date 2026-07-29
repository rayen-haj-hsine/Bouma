//! Sidebar view — drives, quick access, and favorites.

use crate::message::Message;
use crate::theme;
use iced::widget::{button, container, scrollable, text, Column};
use iced::{Element, Length};
use std::path::PathBuf;

/// Renders the sidebar.
pub fn view<'a>(favorites: &'a [PathBuf], current_path: &'a std::path::Path) -> Element<'a, Message> {
    let mut content = Column::new().spacing(theme::SPACING_SM);

    // ── Quick Access ────────────────────────────────────────────
    content = content.push(section_header("Quick Access"));

    let quick_access = [
        ("🏠 Home", dirs_home()),
        ("📄 Documents", dirs_documents()),
        ("⬇ Downloads", dirs_downloads()),
        ("🖼 Pictures", dirs_pictures()),
        ("🎵 Music", dirs_music()),
        ("🎬 Videos", dirs_videos()),
        ("💻 Desktop", dirs_desktop()),
    ];

    for (label, path) in quick_access {
        if let Some(path) = path {
            let is_active = current_path.starts_with(&path);
            content = content.push(sidebar_item(label.to_string(), path, is_active));
        }
    }

    // ── Drives ──────────────────────────────────────────────────
    content = content.push(section_header("Drives"));

    // On Windows, enumerate common drive letters
    for letter in ['C', 'D', 'E', 'F'] {
        let drive_path = PathBuf::from(format!("{letter}:\\"));
        if drive_path.exists() {
            let label = format!("💿 {letter}:");
            let is_active = current_path.starts_with(&drive_path);
            content = content.push(sidebar_item(label, drive_path, is_active));
        }
    }

    // ── Favorites ───────────────────────────────────────────────
    if !favorites.is_empty() {
        content = content.push(section_header("Favorites"));
        for fav in favorites {
            let label = fav
                .file_name()
                .map(|n| format!("⭐ {}", n.to_string_lossy()))
                .unwrap_or_else(|| "⭐ ???".to_string());
            let is_active = current_path.starts_with(fav);
            content = content.push(sidebar_item(label, fav.clone(), is_active));
        }
    }

    container(scrollable(content.width(Length::Fill)).height(Length::Fill))
        .width(theme::SIDEBAR_WIDTH)
        .height(Length::Fill)
        .padding([theme::PADDING, theme::PADDING_SM])
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

/// Section header label (e.g., "Quick Access", "Drives").
fn section_header<'a>(label: &'static str) -> Element<'a, Message> {
    container(text(label).size(11).color(theme::TEXT_MUTED))
        .padding(iced::Padding {
            top: theme::PADDING,
            right: 0.0,
            bottom: theme::SPACING_SM,
            left: 0.0,
        })
        .into()
}

/// A single clickable sidebar item.
fn sidebar_item<'a>(label: String, path: PathBuf, is_active: bool) -> Element<'a, Message> {
    let bg = if is_active {
        theme::BG_SELECTED
    } else {
        iced::Color::TRANSPARENT
    };

    button(
        text(label)
            .size(13)
            .color(if is_active {
                theme::ACCENT
            } else {
                theme::TEXT_PRIMARY
            }),
    )
    .on_press(Message::SidebarNavigate(path))
    .width(Length::Fill)
    .padding([6, 10])
    .style(move |_theme, status| {
        let bg_color = match status {
            button::Status::Hovered if !is_active => theme::BG_ELEVATED,
            _ => bg,
        };
        button::Style {
            background: Some(iced::Background::Color(bg_color)),
            text_color: theme::TEXT_PRIMARY,
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        }
    })
    .into()
}

// ── Platform directory helpers ──────────────────────────────────

fn dirs_home() -> Option<PathBuf> {
    directories::UserDirs::new().map(|d| d.home_dir().to_path_buf())
}
fn dirs_documents() -> Option<PathBuf> {
    directories::UserDirs::new().and_then(|d| d.document_dir().map(|p| p.to_path_buf()))
}
fn dirs_downloads() -> Option<PathBuf> {
    directories::UserDirs::new().and_then(|d| d.download_dir().map(|p| p.to_path_buf()))
}
fn dirs_pictures() -> Option<PathBuf> {
    directories::UserDirs::new().and_then(|d| d.picture_dir().map(|p| p.to_path_buf()))
}
fn dirs_music() -> Option<PathBuf> {
    directories::UserDirs::new().and_then(|d| d.audio_dir().map(|p| p.to_path_buf()))
}
fn dirs_videos() -> Option<PathBuf> {
    directories::UserDirs::new().and_then(|d| d.video_dir().map(|p| p.to_path_buf()))
}
fn dirs_desktop() -> Option<PathBuf> {
    directories::UserDirs::new().and_then(|d| d.desktop_dir().map(|p| p.to_path_buf()))
}
