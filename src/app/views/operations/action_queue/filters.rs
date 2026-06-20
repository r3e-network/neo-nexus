use eframe::egui;

use crate::diagnostics::{CheckSeverity, FleetDiagnostics};

use super::super::super::super::{theme::muted_text, NeoNexusApp};

pub(super) fn render_action_filters(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    diagnostics: &FleetDiagnostics,
) {
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new("Severity").color(muted_text()));
        severity_button(app, ui, "All", None);
        severity_button(app, ui, "Critical", Some(CheckSeverity::Critical));
        severity_button(app, ui, "Warning", Some(CheckSeverity::Warning));
        ui.separator();
        if ui
            .add_enabled(
                diagnostics.critical_count > 0,
                egui::Button::new("Focus Critical"),
            )
            .on_hover_text("Show critical readiness actions and select the highest-risk row")
            .clicked()
        {
            app.focus_action_queue_severity(diagnostics, CheckSeverity::Critical);
        }
        if ui
            .add_enabled(
                diagnostics.warning_count > 0,
                egui::Button::new("Focus Warning"),
            )
            .on_hover_text("Show warning readiness actions and select the highest-risk row")
            .clicked()
        {
            app.focus_action_queue_severity(diagnostics, CheckSeverity::Warning);
        }
        if ui
            .add_enabled(
                app.has_active_action_queue_filter(),
                egui::Button::new("Clear Filters"),
            )
            .on_hover_text("Show all readiness actions")
            .clicked()
        {
            app.clear_action_queue_filters(diagnostics);
        }
    });
    let response = ui.add_sized(
        [ui.available_width(), 24.0],
        egui::TextEdit::singleline(&mut app.action_queue_query).hint_text("Search"),
    );
    if response.changed() {
        app.action_queue_page = 0;
    }
    ui.separator();
}

fn severity_button(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    label: &str,
    severity: Option<CheckSeverity>,
) {
    if ui
        .selectable_label(app.action_queue_severity_filter == severity, label)
        .clicked()
    {
        app.action_queue_severity_filter = severity;
        app.action_queue_page = 0;
    }
}
