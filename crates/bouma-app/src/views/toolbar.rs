//! Toolbar view — back/forward buttons, breadcrumb path, type filter dropdown, search bar.

use crate::message::{Message, ViewMode};
use crate::theme;
use bouma_core::entry::FileTypeFilter;
use iced::widget::{button, container, pick_list, row, text, text_input};
use iced::{Element, Length};

/// Renders the toolbar row.
pub fn view<'a>(
    current_path: &str,
    can_go_back: bool,
    can_go_forward: bool,
    search_text: &str,
    selected_type_filter: FileTypeFilter,
    view_mode: ViewMode,
) -> Element<'a, Message> {
    let back_btn = nav_button("←", Message::GoBack, can_go_back);
    let forward_btn = nav_button("→", Message::GoForward, can_go_forward);
    let up_btn = nav_button("↑", Message::GoUp, true);

    let (mode_label, next_mode) = match view_mode {
        ViewMode::MindMap => ("🗺️ Map View", ViewMode::ListView),
        ViewMode::ListView => ("📜 List View", ViewMode::MindMap),
    };

    let mode_btn = button(text(mode_label).size(12).color(theme::TEXT_PRIMARY))
        .on_press(Message::SetViewMode(next_mode))
        .padding([4, 10])
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

    let breadcrumb = container(
        text(current_path.to_string())
            .size(14)
            .color(theme::TEXT_PRIMARY),
    )
    .padding(theme::PADDING_SM)
    .width(Length::Fill);

    let type_selector = pick_list(
        FileTypeFilter::ALL,
        Some(selected_type_filter),
        Message::FilterTypeSelected,
    )
    .text_size(12)
    .padding([4, 8]);

    let search_input = text_input("Search files & subfolders...", search_text)
        .on_input(Message::SearchInputChanged)
        .on_submit(Message::SearchSubmit)
        .size(13)
        .width(200);

    let search_widget: Element<'a, Message> = if !search_text.is_empty() {
        row![
            type_selector,
            search_input,
            button(text("✕").size(12).color(theme::TEXT_MUTED))
                .on_press(Message::SearchClear)
                .padding([4, 8])
                .style(|_theme, status| {
                    let bg = match status {
                        button::Status::Hovered => theme::BG_ELEVATED,
                        _ => iced::Color::TRANSPARENT,
                    };
                    button::Style {
                        background: Some(iced::Background::Color(bg)),
                        text_color: theme::TEXT_PRIMARY,
                        border: iced::Border {
                            color: iced::Color::TRANSPARENT,
                            width: 0.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    }
                })
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center)
        .into()
    } else {
        row![type_selector, search_input]
            .spacing(4)
            .align_y(iced::Alignment::Center)
            .into()
    };

    container(
        row![back_btn, forward_btn, up_btn, mode_btn, breadcrumb, search_widget]
            .spacing(theme::SPACING_SM)
            .align_y(iced::Alignment::Center),
    )
    .padding(theme::PADDING_SM)
    .height(theme::TOOLBAR_HEIGHT)
    .width(Length::Fill)
    .style(|_theme| container::Style {
        background: Some(iced::Background::Color(theme::BG_SURFACE)),
        border: iced::Border {
            color: theme::BORDER,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}

/// Creates a small navigation button.
fn nav_button(label: &str, msg: Message, enabled: bool) -> Element<'_, Message> {
    let btn = button(
        text(label)
            .size(16)
            .color(if enabled {
                theme::TEXT_PRIMARY
            } else {
                theme::TEXT_MUTED
            }),
    )
    .padding([4, 10])
    .style(|_theme, status| {
        let bg = match status {
            button::Status::Hovered => theme::BG_ELEVATED,
            button::Status::Pressed => theme::BG_SELECTED,
            _ => iced::Color::TRANSPARENT,
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: theme::TEXT_PRIMARY,
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        }
    });

    if enabled {
        btn.on_press(msg).into()
    } else {
        btn.into()
    }
}
