mod filters;
mod table;

use eframe::egui;

use crate::app::domain::FleetDiagnostics;

use super::super::super::{
    paging::page_count,
    theme,
    views::NodeWorkspaceTab,
    widgets::{empty_state, empty_state_with_action, pagination_bar},
    NeoNexusApp, PORT_MATRIX_PAGE_SIZE,
};
use filters::render_port_filters;
use table::{render_port_table, render_selected_port_summary};

impl NeoNexusApp {
    pub(super) fn render_port_matrix(&mut self, ui: &mut egui::Ui, diagnostics: &FleetDiagnostics) {
        if self.fleet.nodes.is_empty() {
            if empty_state_with_action(
                ui,
                "No ports",
                "Create a node to inspect RPC, P2P, and WebSocket bindings.",
                Some("Create node"),
            ) {
                self.open_node_workspace_tab(NodeWorkspaceTab::Studio);
            }
            return;
        }

        render_port_filters(self, ui, diagnostics);
        self.clamp_port_matrix_page(diagnostics);
        let rows = self.filtered_port_matrix_rows(diagnostics);
        if rows.is_empty() {
            empty_state(ui, "No matching ports", "Adjust the port matrix filter.");
            return;
        }
        self.ensure_visible_port_matrix_selection(&rows);

        let total_pages = page_count(rows.len(), PORT_MATRIX_PAGE_SIZE);
        self.operations_ui.port_matrix_page =
            self.operations_ui.port_matrix_page.min(total_pages - 1);
        pagination_bar(
            ui,
            &mut self.operations_ui.port_matrix_page,
            total_pages,
            rows.len(),
        );
        ui.add_space(theme::SM);

        let start = self.operations_ui.port_matrix_page * PORT_MATRIX_PAGE_SIZE;
        render_port_table(self, ui, &rows, start);
        render_selected_port_summary(self, ui, &rows);

    }
}
