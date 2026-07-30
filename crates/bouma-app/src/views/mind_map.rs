//! Mind Map View — interactive visual diagram of system directories
//! allowing users to expand folders under C:\ and prune closed folders from search.

use crate::message::{Message, ViewMode};
use crate::theme;
use bouma_core::entry::{EntryKind, FileTypeFilter};
use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input, Column};
use iced::{Color, Element, Length};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Renders the Mind Map hero landing interface.
pub fn view<'a>(
    current_path: &'a Path,
    closed_folders: &'a HashSet<PathBuf>,
    expanded_folders: &'a HashSet<PathBuf>,
    search_text: &'a str,
    selected_type_filter: FileTypeFilter,
) -> Element<'a, Message> {
    // ── 1. Hero Search Header ─────────────────────────────────────
    let title = text("Bouma")
        .size(28)
        .color(theme::ACCENT);

    let subtitle = text("Interactive System Mind Map — Expand folders to explore, mark 'Closed' to prune from search")
        .size(12)
        .color(theme::TEXT_MUTED);

    let type_selector = pick_list(
        FileTypeFilter::ALL,
        Some(selected_type_filter),
        Message::FilterTypeSelected,
    )
    .text_size(12)
    .padding([6, 10]);

    let search_input = text_input("Type to search system (pruned folders skipped)...", search_text)
        .on_input(Message::SearchInputChanged)
        .on_submit(Message::SearchSubmit)
        .size(14)
        .padding([8, 12])
        .width(420);

    let search_box = row![type_selector, search_input]
        .spacing(8)
        .align_y(iced::Alignment::Center);

    let switch_to_list = button(
        text("📜 Switch to Explorer View")
            .size(12)
            .color(theme::TEXT_PRIMARY),
    )
    .on_press(Message::SetViewMode(ViewMode::ListView))
    .padding([6, 12])
    .style(|_theme, status| {
        let bg = match status {
            button::Status::Hovered => theme::BG_ELEVATED,
            _ => theme::BG_SURFACE,
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: theme::TEXT_PRIMARY,
            border: iced::Border {
                color: theme::BORDER,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        }
    });

    let hero_header = column![
        title,
        subtitle,
        row![search_box, switch_to_list]
            .spacing(16)
            .align_y(iced::Alignment::Center),
    ]
    .spacing(12)
    .align_x(iced::Alignment::Center);

    // ── 2. Mind Map Tree Canvas ────────────────────────────────────
    let root_path_buf = current_path
        .ancestors()
        .last()
        .unwrap_or(current_path)
        .to_path_buf();

    let root_is_closed = closed_folders.contains(&root_path_buf);
    let root_card = root_node_card(root_path_buf, root_is_closed, closed_folders.len());

    let default_nodes = get_system_nodes(current_path);
    let mut tree_column: Column<'a, Message> = Column::new().spacing(6);

    for node_path in default_nodes {
        tree_column = tree_column.push(render_tree_node(node_path, closed_folders, expanded_folders, 0));
    }

    let map_canvas = column![
        root_card,
        text("│").size(16).color(theme::BORDER),
        text("▼").size(12).color(theme::ACCENT),
        tree_column,
    ]
    .spacing(8)
    .align_x(iced::Alignment::Center);

    let content = column![hero_header, map_canvas]
        .spacing(24)
        .align_x(iced::Alignment::Center)
        .width(Length::Fill);

    container(scrollable(content).height(Length::Fill))
        .padding(theme::PADDING)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(theme::BG_BASE)),
            ..Default::default()
        })
        .into()
}

/// Recursively renders a node card and any expanded children.
/// Uses owned `PathBuf` to avoid borrow-escaping-scope issues.
fn render_tree_node<'a>(
    path: PathBuf,
    closed_folders: &'a HashSet<PathBuf>,
    expanded_folders: &'a HashSet<PathBuf>,
    depth: usize,
) -> Element<'a, Message> {
    let is_closed = closed_folders.contains(&path);
    let is_expanded = expanded_folders.contains(&path);

    let mut col: Column<'a, Message> = Column::new().spacing(4);
    col = col.push(node_card(path.clone(), is_closed, is_expanded, depth));

    if is_expanded && !is_closed {
        if let Ok(entries) = bouma_filesystem::read_directory(&path) {
            let subdirs: Vec<PathBuf> = entries
                .into_iter()
                .filter(|e| {
                    e.kind == EntryKind::Directory
                        && (!e.hidden || path.components().count() <= 3)
                })
                .map(|e| e.path)
                .collect();

            let mut sub_col: Column<'a, Message> = Column::new().spacing(4);

            if subdirs.is_empty() {
                let left_pad = 12.0 + ((depth + 1) * 20) as f32;
                sub_col = sub_col.push(
                    container(
                        text("  (No subdirectories)")
                            .size(11)
                            .color(theme::TEXT_MUTED),
                    )
                    .padding(iced::Padding {
                        top: 2.0,
                        right: 0.0,
                        bottom: 2.0,
                        left: left_pad,
                    }),
                );
            } else {
                for dir_path in subdirs.into_iter().take(15) {
                    sub_col = sub_col.push(render_tree_node(
                        dir_path,
                        closed_folders,
                        expanded_folders,
                        depth + 1,
                    ));
                }
            }
            col = col.push(sub_col);
        }
    }

    col.into()
}

fn root_node_card(root_path: PathBuf, is_closed: bool, pruned_count: usize) -> Element<'static, Message> {
    let name = root_path.to_string_lossy().into_owned();
    let status_label = if is_closed { "PRUNED" } else { "ACTIVE ROOT" };
    let status_color = if is_closed {
        Color { r: 0.90, g: 0.30, b: 0.30, a: 1.0 }
    } else {
        Color { r: 0.35, g: 0.85, b: 0.60, a: 1.0 }
    };

    let pruned_text = if pruned_count > 0 {
        format!(" ({pruned_count} subfolders pruned)")
    } else {
        String::new()
    };

    container(
        row![
            text("💻 ").size(20),
            column![
                text(format!("{name} Root Drive"))
                    .size(16)
                    .color(theme::TEXT_PRIMARY),
                text(format!("{status_label}{pruned_text}"))
                    .size(11)
                    .color(status_color),
            ]
            .spacing(2),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
    )
    .padding([12, 20])
    .style(move |_theme| container::Style {
        background: Some(iced::Background::Color(theme::BG_SURFACE)),
        border: iced::Border {
            color: status_color,
            width: 2.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn node_card(path: PathBuf, is_closed: bool, is_expanded: bool, depth: usize) -> Element<'static, Message> {
    let folder_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    let toggle_expand_path = path.clone();
    let toggle_close_path = path.clone();
    let open_dir_path = path.clone();

    let expand_label = if is_expanded { "▼ " } else { "▶ " };
    let expand_btn = button(text(expand_label).size(12).color(theme::ACCENT))
        .on_press(Message::ToggleFolderExpanded(toggle_expand_path))
        .padding([4, 6])
        .style(|_theme, status| {
            let bg = match status {
                button::Status::Hovered => theme::BG_ELEVATED,
                _ => iced::Color::TRANSPARENT,
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: theme::ACCENT,
                border: iced::Border {
                    color: iced::Color::TRANSPARENT,
                    width: 0.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }
        });

    let (status_text, status_color, btn_label, icon) = if is_closed {
        ("CLOSED (PRUNED FROM SEARCH)", Color { r: 0.90, g: 0.35, b: 0.35, a: 1.0 }, "🔓 Open & Search", "🔒")
    } else {
        ("ACTIVE (SEARCHABLE)", Color { r: 0.35, g: 0.85, b: 0.60, a: 1.0 }, "🔒 Close & Prune", "📁")
    };

    let toggle_btn = button(text(btn_label).size(11))
        .on_press(Message::ToggleFolderClosed(toggle_close_path))
        .padding([4, 10])
        .style(move |_theme, status| {
            let bg = match status {
                button::Status::Hovered => theme::BG_ELEVATED,
                _ => iced::Color::TRANSPARENT,
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: status_color,
                border: iced::Border {
                    color: status_color,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }
        });

    let open_btn = button(text("📂 Browse").size(11))
        .on_press(Message::OpenDirectory(open_dir_path))
        .padding([4, 10])
        .style(|_theme, status| {
            let bg = match status {
                button::Status::Hovered => theme::BG_ELEVATED,
                _ => iced::Color::TRANSPARENT,
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: theme::TEXT_PRIMARY,
                border: iced::Border {
                    color: theme::BORDER,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }
        });

    let left_padding = 12.0 + (depth * 24) as f32;

    container(
        row![
            expand_btn,
            text(icon).size(16),
            column![
                text(folder_name).size(13).color(theme::TEXT_PRIMARY),
                text(status_text).size(10).color(status_color),
            ]
            .spacing(2)
            .width(Length::Fixed(200.0)),
            row![toggle_btn, open_btn].spacing(8),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
    )
    .padding(iced::Padding {
        top: 6.0,
        right: 12.0,
        bottom: 6.0,
        left: left_padding,
    })
    .style(move |_theme| container::Style {
        background: Some(iced::Background::Color(theme::BG_SURFACE)),
        border: iced::Border {
            color: if is_closed {
                Color { r: 0.40, g: 0.20, b: 0.20, a: 1.0 }
            } else {
                theme::BORDER
            },
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    })
    .into()
}

/// Helper to resolve common system nodes for display in the mind map.
fn get_system_nodes(current_path: &Path) -> Vec<PathBuf> {
    let mut nodes = Vec::new();
    let root = current_path.ancestors().last().unwrap_or(current_path);

    // Common system folders under root (e.g. C:\)
    let candidates = [
        root.join("Users"),
        root.join("Program Files"),
        root.join("Program Files (x86)"),
        root.join("Windows"),
    ];

    for candidate in candidates {
        if candidate.exists() {
            nodes.push(candidate);
        }
    }

    // Also include user folders if we can find user profile
    if let Some(user_dirs) = directories::UserDirs::new() {
        let home = user_dirs.home_dir();
        for sub in &["Desktop", "Documents", "Downloads", "Pictures", "Videos", "Music"] {
            let path = home.join(sub);
            if path.exists() && !nodes.contains(&path) {
                nodes.push(path);
            }
        }
    }

    nodes
}
