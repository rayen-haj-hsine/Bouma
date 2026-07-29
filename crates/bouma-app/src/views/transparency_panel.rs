//! Transparency System View — displays operation progress, speed, ETA, and timing diagnostics.

use crate::message::Message;
use crate::theme;
use bouma_core::operations::{OperationDiagnostics, OperationProgress};
use iced::widget::{column, container, progress_bar, row, text};
use iced::{Element, Length};

/// Renders the Transparency Panel.
pub fn view<'a>(
    progress: Option<&OperationProgress>,
    diagnostics: Option<&OperationDiagnostics>,
) -> Element<'a, Message> {
    if progress.is_none() && diagnostics.is_none() {
        return container(column![]).into();
    }

    let mut panel = column![].spacing(theme::SPACING_SM);

    // ── Operation Progress ──────────────────────────────────────
    if let Some(prog) = progress {
        let title = format!("{} {}", prog.kind.label(), prog.source.display());
        let percent = prog.percent();
        let fraction = prog.fraction();

        // Calculate speed & ETA if bytes are tracked
        let status_text = format!("{percent}% complete ({} / {})", prog.bytes_done, prog.total_bytes);

        panel = panel.push(text(title).size(13).color(theme::TEXT_PRIMARY));
        panel = panel.push(progress_bar(0.0..=1.0, fraction));
        panel = panel.push(text(status_text).size(11).color(theme::TEXT_SECONDARY));
    }

    // ── Diagnostic Information ──────────────────────────────────
    if let Some(diag) = diagnostics {
        let label_text = text(format!("Diagnostics: {}", diag.label))
            .size(12)
            .color(theme::ACCENT);

        let mut diag_row = row![label_text].spacing(theme::SPACING);

        for (phase, duration) in &diag.phases {
            let phase_text = text(format!("{phase}: {}ms", duration.as_millis()))
                .size(11)
                .color(theme::TEXT_MUTED);
            diag_row = diag_row.push(phase_text);
        }

        let total_text = text(format!("Total: {}ms", diag.elapsed().as_millis()))
            .size(11)
            .color(theme::TEXT_SECONDARY);
        diag_row = diag_row.push(total_text);

        panel = panel.push(diag_row);
    }

    container(panel)
        .padding(theme::PADDING_SM)
        .width(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(theme::BG_SURFACE)),
            border: iced::Border {
                color: theme::ACCENT,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into()
}
