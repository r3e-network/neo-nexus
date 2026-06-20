use eframe::egui;

use crate::diagnostics::{CheckSeverity, FleetDiagnostics, ReadinessAction};

use super::{
    super::super::{
        paging::page_count,
        text::truncate_middle,
        theme::muted_text,
        widgets::{empty_state, pagination_bar},
        NeoNexusApp, ACTION_QUEUE_PAGE_SIZE,
    },
    helpers::{score_color, severity_color},
};

impl NeoNexusApp {
    pub(super) fn render_action_queue(
        &mut self,
        ui: &mut egui::Ui,
        diagnostics: &FleetDiagnostics,
    ) {
        if diagnostics.nodes.is_empty() {
            empty_state(
                ui,
                "No nodes",
                "Create a node before running readiness checks.",
            );
            ui.add_space(8.0);
            if ui.button("Export Report").clicked() {
                self.export_workspace_readiness_report(diagnostics);
            }
            return;
        }

        render_action_filters(self, ui);
        self.clamp_action_queue_page(diagnostics);
        let actions = self.filtered_readiness_actions(diagnostics);
        if actions.is_empty() {
            empty_state(ui, "No matching actions", "Adjust the action filter.");
            render_export_action(self, ui, diagnostics);
            return;
        }

        let total_pages = page_count(actions.len(), ACTION_QUEUE_PAGE_SIZE);
        self.action_queue_page = self.action_queue_page.min(total_pages - 1);
        pagination_bar(ui, &mut self.action_queue_page, total_pages, actions.len());
        ui.separator();

        let start = self.action_queue_page * ACTION_QUEUE_PAGE_SIZE;
        render_action_table(self, ui, &actions, start);
        render_export_action(self, ui, diagnostics);
    }
}

fn render_action_filters(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Severity").color(muted_text()));
        severity_button(app, ui, "All", None);
        severity_button(app, ui, "Critical", Some(CheckSeverity::Critical));
        severity_button(app, ui, "Warning", Some(CheckSeverity::Warning));
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

fn render_action_table(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    actions: &[ReadinessAction],
    start: usize,
) {
    egui::Grid::new("operations_action_queue")
        .striped(true)
        .min_col_width(62.0)
        .show(ui, |ui| {
            ui.strong("Severity");
            ui.strong("Node");
            ui.strong("Score");
            ui.strong("Check");
            ui.strong("Detail");
            ui.end_row();

            for action in actions.iter().skip(start).take(ACTION_QUEUE_PAGE_SIZE) {
                render_action_row(app, ui, action);
                ui.end_row();
            }
        });
}

fn render_action_row(app: &mut NeoNexusApp, ui: &mut egui::Ui, action: &ReadinessAction) {
    ui.label(
        egui::RichText::new(action.severity.label())
            .strong()
            .color(severity_color(action.severity)),
    );
    let selected = app.selected_node.as_deref() == Some(action.node_id.as_str());
    if ui
        .selectable_label(selected, truncate_middle(&action.node_name, 18))
        .clicked()
    {
        app.selected_node = Some(action.node_id.clone());
    }
    ui.label(
        egui::RichText::new(format!("{}%", action.node_score))
            .color(score_color(action.node_score)),
    );
    ui.label(truncate_middle(&action.title, 18));
    ui.label(truncate_middle(&action.detail, 44));
}

fn render_export_action(app: &mut NeoNexusApp, ui: &mut egui::Ui, diagnostics: &FleetDiagnostics) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ui.button("Export Report").clicked() {
            app.export_workspace_readiness_report(diagnostics);
        }
        ui.label(egui::RichText::new("Writes text and JSON evidence.").color(muted_text()));
    });
}
