use eframe::egui;

use crate::app::domain::{CheckSeverity, FleetDiagnostics};

use super::super::super::super::{
    theme,
    widgets::{chip_pill, filter_bar, filter_chip, secondary_button_enabled},
    NeoNexusApp,
};
use super::super::helpers::{resolution_filter_combo, severity_filter_label};

pub(super) fn render_action_filters(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    diagnostics: &FleetDiagnostics,
) {
    let resolution_counts = app.action_queue_resolution_counts(diagnostics);
    let severity_counts = app.action_queue_severity_counts(diagnostics);
    ui.horizontal_wrapped(|ui| {
        ui.label(theme::muted_body("Severity"));
        chip_pill(ui, |ui| {
            severity_button(app, ui, "All", None, &severity_counts);
            severity_button(
                app,
                ui,
                "Critical",
                Some(CheckSeverity::Critical),
                &severity_counts,
            );
            severity_button(
                app,
                ui,
                "Warning",
                Some(CheckSeverity::Warning),
                &severity_counts,
            );
        });
        ui.separator();
        if resolution_filter_combo(
            ui,
            "action_queue_resolution_filter",
            &mut app.action_queue_resolution_filter,
            &resolution_counts,
        ) {
            app.set_action_queue_resolution_filter(diagnostics, app.action_queue_resolution_filter);
        }
        ui.separator();
        if secondary_button_enabled(ui, "Focus Critical", diagnostics.critical_count > 0)
            .on_hover_text("Show critical readiness actions and select the highest-risk row")
            .clicked()
        {
            app.focus_action_queue_severity(diagnostics, CheckSeverity::Critical);
        }
        if secondary_button_enabled(ui, "Focus Warning", diagnostics.warning_count > 0)
            .on_hover_text("Show warning readiness actions and select the highest-risk row")
            .clicked()
        {
            app.focus_action_queue_severity(diagnostics, CheckSeverity::Warning);
        }
        if secondary_button_enabled(ui, "Clear Filters", app.has_active_action_queue_filter())
            .on_hover_text("Show all readiness actions")
            .clicked()
        {
            app.clear_action_queue_filters(diagnostics);
        }
    });
    ui.add_space(theme::XS);
    if filter_bar(ui, &mut app.action_queue_query, "Search actions") {
        app.action_queue_page = 0;
    }
    ui.add_space(theme::SM);
}

fn severity_button(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    label: &str,
    severity: Option<CheckSeverity>,
    counts: &[(CheckSeverity, usize)],
) {
    if filter_chip(
        ui,
        &severity_filter_label(label, severity, counts),
        app.action_queue_severity_filter == severity,
    ) {
        app.action_queue_severity_filter = severity;
        app.action_queue_page = 0;
    }
}
