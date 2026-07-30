//! Transparency System View — real-time operation progress, timing diagnostics,
//! and search scan statistics.

use crate::message::{Message, SearchStats};
use crate::theme;
use bouma_core::operations::{OperationDiagnostics, OperationProgress};
use iced::widget::{column, container, progress_bar, row, text};
use iced::{Color, Element, Length};

/// Renders the Transparency Panel.
///
/// Shows one or more of:
/// - "Searching…" indicator while a recursive scan is in flight
/// - Active file-operation progress bar + ETA
/// - Folder load timing diagnostics
/// - Search scan statistics (query, tier breakdown, depth, duration)
pub fn view<'a>(
    is_loading: bool,
    progress: Option<&OperationProgress>,
    diagnostics: Option<&OperationDiagnostics>,
    search_stats: Option<&SearchStats>,
) -> Element<'a, Message> {
    let has_content = is_loading || progress.is_some() || diagnostics.is_some() || search_stats.is_some();
    if !has_content {
        return container(column![]).into();
    }

    let mut panel = column![].spacing(6);

    // ── Searching indicator ─────────────────────────────────────
    if is_loading {
        panel = panel.push(
            row![
                text("⟳ ").size(13).color(theme::ACCENT),
                text("Searching…").size(12).color(theme::ACCENT),
                text("  scanning files, please wait")
                    .size(11)
                    .color(theme::TEXT_MUTED),
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center),
        );
    }

    // ── Active file-operation progress ──────────────────────────
    if let Some(prog) = progress {
        let title = format!("{} {}", prog.kind.label(), prog.source.display());
        let percent = prog.percent();
        let fraction = prog.fraction();
        let status_text = format!("{percent}% — {} / {}", prog.bytes_done, prog.total_bytes);

        panel = panel.push(
            row![
                text("⚙ ").size(12).color(theme::ACCENT),
                text(title).size(12).color(theme::TEXT_PRIMARY),
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center),
        );
        panel = panel.push(progress_bar(0.0..=1.0, fraction));
        panel = panel.push(text(status_text).size(10).color(theme::TEXT_SECONDARY));
    }

    // ── Folder load diagnostics ─────────────────────────────────
    if let Some(diag) = diagnostics {
        let total_ms = diag.elapsed().as_millis();
        let mut diag_row = row![
            text("⏱ ").size(11).color(theme::TEXT_MUTED),
            text(format!("Folder loaded in {}ms", total_ms))
                .size(11)
                .color(theme::TEXT_SECONDARY),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center);

        for (phase, duration) in &diag.phases {
            diag_row = diag_row.push(
                text(format!("  │  {phase}: {}ms", duration.as_millis()))
                    .size(10)
                    .color(theme::TEXT_MUTED),
            );
        }
        panel = panel.push(diag_row);
    }

    // ── Search scan statistics ──────────────────────────────────
    if let Some(stats) = search_stats {
        if !stats.query.is_empty() {
            let total_results: usize = stats.tier_counts.iter().sum();

            // Header row: query + high-level numbers
            panel = panel.push(
                row![
                    text("🔍 ").size(11).color(theme::ACCENT),
                    text(format!("\"{}\"", stats.query))
                        .size(12)
                        .color(theme::TEXT_PRIMARY),
                    text(format!(
                        "  →  {} result{} from {} scanned  ·  {}ms  ·  depth {}",
                        total_results,
                        if total_results == 1 { "" } else { "s" },
                        fmt_count(stats.total_scanned),
                        stats.scan_ms,
                        stats.depth_used,
                    ))
                    .size(11)
                    .color(theme::TEXT_MUTED),
                ]
                .spacing(4)
                .align_y(iced::Alignment::Center),
            );

            // Tier breakdown bar
            if total_results > 0 {
                let tier_info: [(usize, &str, Color); 4] = [
                    (stats.tier_counts[0], "Exact",   Color { r: 0.35, g: 0.85, b: 0.60, a: 1.0 }),
                    (stats.tier_counts[1], "Prefix",  Color { r: 0.45, g: 0.70, b: 1.00, a: 1.0 }),
                    (stats.tier_counts[2], "Word",    Color { r: 0.90, g: 0.75, b: 0.30, a: 1.0 }),
                    (stats.tier_counts[3], "Partial", Color { r: 0.65, g: 0.65, b: 0.65, a: 1.0 }),
                ];

                let mut tier_row = row![].spacing(12).align_y(iced::Alignment::Center);
                for (count, label, color) in tier_info {
                    if count > 0 {
                        let dot_color = color;
                        tier_row = tier_row.push(
                            row![
                                // Small coloured dot
                                container(column![])
                                    .width(8)
                                    .height(8)
                                    .style(move |_theme| container::Style {
                                        background: Some(iced::Background::Color(dot_color)),
                                        border: iced::Border {
                                            radius: 4.0.into(),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }),
                                text(format!(" {label}: {count}"))
                                    .size(10)
                                    .color(color),
                            ]
                            .spacing(2)
                            .align_y(iced::Alignment::Center),
                        );
                    }
                }
                panel = panel.push(tier_row);
            }
        }
    }

    container(panel)
        .padding([6.0, theme::PADDING_SM])
        .width(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(Color {
                r: 0.09,
                g: 0.09,
                b: 0.11,
                a: 1.0,
            })),
            border: iced::Border {
                color: theme::BORDER,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

/// Formats large counts with K suffix (e.g. 12345 → "12.3k").
fn fmt_count(n: usize) -> String {
    if n >= 1000 {
        format!("{:.1}k", n as f64 / 1000.0)
    } else {
        n.to_string()
    }
}
