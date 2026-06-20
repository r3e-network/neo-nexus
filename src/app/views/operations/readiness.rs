use eframe::egui;

use crate::diagnostics::{CheckSeverity, DiagnosticCheck, FleetDiagnostics};

use super::{
    super::super::{
        paging::page_count,
        text::truncate_middle,
        theme::muted_text,
        view::View,
        widgets::{empty_state, fact, pagination_bar},
        NeoNexusApp, READINESS_CHECK_PAGE_SIZE,
    },
    helpers::{score_color, severity_color},
};

impl NeoNexusApp {
    pub(super) fn render_selected_readiness(
        &mut self,
        ui: &mut egui::Ui,
        diagnostics: &FleetDiagnostics,
    ) {
        let Some(selected_id) = self.selected_node.as_deref() else {
            empty_state(ui, "No selection", "Select a node from Inventory.");
            return;
        };
        let Some(node) = diagnostics
            .nodes
            .iter()
            .find(|diagnostic| diagnostic.node_id == selected_id)
            .cloned()
        else {
            empty_state(ui, "No diagnostics", "Reload the workspace.");
            return;
        };

        ui.horizontal(|ui| {
            ui.heading(truncate_middle(&node.node_name, 28));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!("{}%", node.score))
                        .strong()
                        .color(score_color(node.score)),
                );
            });
        });
        fact(ui, "Critical", &node.critical_count().to_string());
        fact(ui, "Warnings", &node.warning_count().to_string());
        ui.separator();

        render_check_filters(self, ui);
        self.clamp_readiness_check_page(&node);
        let checks = self.filtered_readiness_checks(&node);
        render_checks(self, ui, &checks);

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if ui.button("Node Studio").clicked() {
                self.selected_view = View::Nodes;
            }
            if ui.button("Plugins").clicked() {
                self.selected_view = View::Plugins;
            }
        });
    }
}

fn render_check_filters(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Severity").color(muted_text()));
        severity_button(app, ui, "All", None);
        severity_button(app, ui, "Critical", Some(CheckSeverity::Critical));
        severity_button(app, ui, "Warning", Some(CheckSeverity::Warning));
        severity_button(app, ui, "Info", Some(CheckSeverity::Info));
        severity_button(app, ui, "Pass", Some(CheckSeverity::Pass));
    });
    let response = ui.add_sized(
        [ui.available_width(), 24.0],
        egui::TextEdit::singleline(&mut app.readiness_check_query).hint_text("Search"),
    );
    if response.changed() {
        app.readiness_check_page = 0;
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
        .selectable_label(app.readiness_check_severity_filter == severity, label)
        .clicked()
    {
        app.readiness_check_severity_filter = severity;
        app.readiness_check_page = 0;
    }
}

fn render_checks(app: &mut NeoNexusApp, ui: &mut egui::Ui, checks: &[DiagnosticCheck]) {
    if checks.is_empty() {
        empty_state(ui, "No matching checks", "Adjust the readiness filter.");
        return;
    }

    let total_pages = page_count(checks.len(), READINESS_CHECK_PAGE_SIZE);
    app.readiness_check_page = app.readiness_check_page.min(total_pages - 1);
    pagination_bar(ui, &mut app.readiness_check_page, total_pages, checks.len());
    ui.separator();

    let start = app.readiness_check_page * READINESS_CHECK_PAGE_SIZE;
    for check in checks.iter().skip(start).take(READINESS_CHECK_PAGE_SIZE) {
        render_check_row(ui, check);
    }
}

fn render_check_row(ui: &mut egui::Ui, check: &DiagnosticCheck) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(check.severity.label())
                .strong()
                .color(severity_color(check.severity)),
        );
        ui.label(check.title);
        ui.label(truncate_middle(&check.detail, 54));
    });
}
