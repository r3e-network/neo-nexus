mod filters;
mod table;

use eframe::egui;

use crate::app::domain::FleetDiagnostics;

use super::super::super::{
    paging::page_count,
    theme,
    widgets::{empty_state, empty_state_with_action, pagination_bar, secondary_button},
    NeoNexusApp, ACTION_QUEUE_PAGE_SIZE,
};
use filters::render_action_filters;
use table::{render_action_table, render_selected_action_summary};

impl NeoNexusApp {
    pub(super) fn render_action_queue(
        &mut self,
        ui: &mut egui::Ui,
        diagnostics: &FleetDiagnostics,
    ) {
        if diagnostics.nodes.is_empty() {
            if empty_state_with_action(
                ui,
                "No nodes",
                "Create a node before running readiness checks.",
                Some("Create node"),
            ) {
                self.open_node_workspace_tab(crate::app::views::NodeWorkspaceTab::Studio);
            }
            ui.add_space(theme::SM);
            render_export_action(self, ui, diagnostics);
            return;
        }

        render_action_filters(self, ui, diagnostics);
        self.clamp_action_queue_page(diagnostics);
        let actions = self.filtered_readiness_actions(diagnostics);
        if actions.is_empty() {
            empty_state(ui, "No matching actions", "Adjust the action filter.");
            render_export_action(self, ui, diagnostics);
            return;
        }
        self.ensure_visible_readiness_action_selection(&actions);

        let total_pages = page_count(actions.len(), ACTION_QUEUE_PAGE_SIZE);
        self.operations_ui.action_queue_page = self.operations_ui.action_queue_page.min(total_pages - 1);
        pagination_bar(ui, &mut self.operations_ui.action_queue_page, total_pages, actions.len());
        ui.add_space(theme::SM);

        let start = self.operations_ui.action_queue_page * ACTION_QUEUE_PAGE_SIZE;
        render_action_table(self, ui, &actions, start);
        render_selected_action_summary(self, ui, &actions);
        render_export_action(self, ui, diagnostics);
    }
}

fn render_export_action(app: &mut NeoNexusApp, ui: &mut egui::Ui, diagnostics: &FleetDiagnostics) {
    ui.add_space(theme::SM);
    ui.horizontal(|ui| {
        if secondary_button(ui, "Export Report").clicked() {
            app.export_workspace_readiness_report(diagnostics);
        }
        ui.label(theme::muted_body("Writes text and JSON evidence."));
    });
}
