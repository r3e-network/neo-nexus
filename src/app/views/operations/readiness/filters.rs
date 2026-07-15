use eframe::egui;

use crate::app::domain::{CheckSeverity, NodeDiagnostics};
use crate::app::widgets::{chip_pill, secondary_button_enabled};

use super::super::super::super::{theme::muted_text, NeoNexusApp};
use super::super::helpers::{resolution_filter_combo, severity_filter_label};

pub(super) fn render_check_filters(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    node: &NodeDiagnostics,
) {
    let resolution_counts = app.readiness_check_resolution_counts(node);
    let severity_counts = app.readiness_check_severity_counts(node);
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new("Severity").color(muted_text()));
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
            severity_button(app, ui, "Info", Some(CheckSeverity::Info), &severity_counts);
            severity_button(app, ui, "Pass", Some(CheckSeverity::Pass), &severity_counts);
        });
        ui.separator();
        if resolution_filter_combo(
            ui,
            "readiness_check_resolution_filter",
            &mut app.operations_ui.readiness_check_resolution_filter,
            &resolution_counts,
        ) {
            app.set_readiness_check_resolution_filter(node, app.operations_ui.readiness_check_resolution_filter);
        }
        ui.separator();
        if secondary_button_enabled(ui, "Focus Critical", node.critical_count() > 0)
            .on_hover_text("Show critical checks for the selected node")
            .clicked()
        {
            app.focus_readiness_check_severity(node, CheckSeverity::Critical);
        }
        if secondary_button_enabled(ui, "Focus Warning", node.warning_count() > 0)
            .on_hover_text("Show warning checks for the selected node")
            .clicked()
        {
            app.focus_readiness_check_severity(node, CheckSeverity::Warning);
        }
        if secondary_button_enabled(ui, "Clear Filters", app.has_active_readiness_check_filter())
            .on_hover_text("Show all checks for the selected node")
            .clicked()
        {
            app.clear_readiness_check_filters(node);
        }
    });
    let response = ui.add_sized(
        [ui.available_width(), 24.0],
        egui::TextEdit::singleline(&mut app.operations_ui.readiness_check_query).hint_text("Search"),
    );
    if response.changed() {
        app.operations_ui.readiness_check_page = 0;
    }
    ui.separator();
}

fn severity_button(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    label: &str,
    severity: Option<CheckSeverity>,
    counts: &[(CheckSeverity, usize)],
) {
    if ui
        .selectable_label(
            app.operations_ui.readiness_check_severity_filter == severity,
            severity_filter_label(label, severity, counts),
        )
        .clicked()
    {
        app.operations_ui.readiness_check_severity_filter = severity;
        app.operations_ui.readiness_check_page = 0;
    }
}
