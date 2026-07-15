use eframe::egui;

use crate::app::domain::{CheckSeverity, DiagnosticResolution, EventSeverity};
use crate::app::theme;

use super::super::super::theme::muted_text;

pub(super) fn score_color(score: usize) -> egui::Color32 {
    if score >= 90 {
        theme::success()
    } else if score >= 70 {
        theme::warning()
    } else {
        theme::danger()
    }
}

#[allow(dead_code)] // used by readiness rows when not using severity_badge
pub(super) fn severity_color(severity: CheckSeverity) -> egui::Color32 {
    match severity {
        CheckSeverity::Pass => theme::success(),
        CheckSeverity::Info => muted_text(),
        CheckSeverity::Warning => theme::warning(),
        CheckSeverity::Critical => theme::danger(),
    }
}

pub(super) fn severity_filter_label(
    label: &str,
    severity: Option<CheckSeverity>,
    counts: &[(CheckSeverity, usize)],
) -> String {
    severity.map_or_else(
        || format!("{label} ({})", total_severity_count(counts)),
        |severity| format!("{label} ({})", severity_count(counts, severity)),
    )
}

pub(super) fn event_color(severity: EventSeverity) -> egui::Color32 {
    match severity {
        EventSeverity::Info => muted_text(),
        EventSeverity::Warning => theme::warning(),
        EventSeverity::Critical => theme::danger(),
    }
}

pub(super) fn resolution_filter_combo(
    ui: &mut egui::Ui,
    id: &'static str,
    selected: &mut Option<DiagnosticResolution>,
    counts: &[(DiagnosticResolution, usize)],
) -> bool {
    let previous = *selected;
    ui.label(egui::RichText::new("Workspace").color(muted_text()));
    egui::ComboBox::from_id_salt(id)
        .selected_text(resolution_filter_label(*selected, counts))
        .width(170.0)
        .show_ui(ui, |ui| {
            ui.selectable_value(
                selected,
                None,
                format!("All Workspaces ({})", total_resolution_count(counts)),
            );
            for resolution in DiagnosticResolution::ALL {
                ui.selectable_value(
                    selected,
                    Some(resolution),
                    format!(
                        "{} ({})",
                        resolution.label(),
                        resolution_count(counts, resolution)
                    ),
                )
                .on_hover_text(resolution.hint());
            }
        });
    *selected != previous
}

fn resolution_filter_label(
    selected: Option<DiagnosticResolution>,
    counts: &[(DiagnosticResolution, usize)],
) -> String {
    selected.map_or_else(
        || format!("All Workspaces ({})", total_resolution_count(counts)),
        |resolution| {
            format!(
                "{} ({})",
                resolution.label(),
                resolution_count(counts, resolution)
            )
        },
    )
}

fn resolution_count(
    counts: &[(DiagnosticResolution, usize)],
    resolution: DiagnosticResolution,
) -> usize {
    counts
        .iter()
        .find_map(|(counted, count)| (*counted == resolution).then_some(*count))
        .unwrap_or(0)
}

fn total_resolution_count(counts: &[(DiagnosticResolution, usize)]) -> usize {
    counts.iter().map(|(_, count)| count).sum()
}

fn severity_count(counts: &[(CheckSeverity, usize)], severity: CheckSeverity) -> usize {
    counts
        .iter()
        .find_map(|(counted, count)| (*counted == severity).then_some(*count))
        .unwrap_or(0)
}

fn total_severity_count(counts: &[(CheckSeverity, usize)]) -> usize {
    counts.iter().map(|(_, count)| count).sum()
}
