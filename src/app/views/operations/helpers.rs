use eframe::egui;

use crate::{
    diagnostics::{CheckSeverity, DiagnosticResolution},
    events::EventSeverity,
};

use super::super::super::theme::muted_text;

pub(super) fn score_color(score: usize) -> egui::Color32 {
    if score >= 90 {
        egui::Color32::from_rgb(21, 128, 61)
    } else if score >= 70 {
        egui::Color32::from_rgb(202, 138, 4)
    } else {
        egui::Color32::from_rgb(185, 28, 28)
    }
}

pub(super) fn severity_color(severity: CheckSeverity) -> egui::Color32 {
    match severity {
        CheckSeverity::Pass => egui::Color32::from_rgb(21, 128, 61),
        CheckSeverity::Info => muted_text(),
        CheckSeverity::Warning => egui::Color32::from_rgb(202, 138, 4),
        CheckSeverity::Critical => egui::Color32::from_rgb(185, 28, 28),
    }
}

pub(super) fn event_color(severity: EventSeverity) -> egui::Color32 {
    match severity {
        EventSeverity::Info => muted_text(),
        EventSeverity::Warning => egui::Color32::from_rgb(202, 138, 4),
        EventSeverity::Critical => egui::Color32::from_rgb(185, 28, 28),
    }
}

pub(super) fn resolution_filter_combo(
    ui: &mut egui::Ui,
    id: &'static str,
    selected: &mut Option<DiagnosticResolution>,
) -> bool {
    let previous = *selected;
    ui.label(egui::RichText::new("Workspace").color(muted_text()));
    egui::ComboBox::from_id_salt(id)
        .selected_text(selected.map_or("All Workspaces", |resolution| resolution.label()))
        .width(150.0)
        .show_ui(ui, |ui| {
            ui.selectable_value(selected, None, "All Workspaces");
            for resolution in DiagnosticResolution::ALL {
                ui.selectable_value(selected, Some(resolution), resolution.label())
                    .on_hover_text(resolution.hint());
            }
        });
    *selected != previous
}
