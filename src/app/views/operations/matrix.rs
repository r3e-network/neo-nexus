mod filters;
mod table;

use eframe::egui;

use crate::app::domain::FleetDiagnostics;

use super::super::super::{
    paging::{page_count, rows_that_fit},
    theme,
    views::NodeWorkspaceTab,
    widgets::{empty_state, empty_state_with_action, pagination_bar},
    NeoNexusApp, PORT_MATRIX_PAGE_SIZE,
};
use filters::render_port_filters;
use table::{render_port_table, render_selected_port_summary};

/// Pitch of one `egui::Grid` port row, measured at the 1280x820 design window.
/// Ports and badges never wrap, so the row holds this height at any column width.
const PORT_ROW_PITCH: f32 = 39.0;

/// Chrome around the rows: pagination bar plus its trailing `SM` gap (47pt), the
/// grid's column header (36pt), and the selected-port card below (167pt). The
/// card is unconditional — `ensure_visible_port_matrix_selection` always leaves
/// a row selected.
const PORT_CHROME: f32 = 47.0 + 36.0 + 167.0;

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

        // Measured with the filters already drawn and the pagination bar still
        // to come, so the bar is reserved rather than pre-subtracted.
        let page_size = rows_that_fit(ui.available_height(), PORT_ROW_PITCH, PORT_CHROME)
            .min(PORT_MATRIX_PAGE_SIZE);
        let total_pages = page_count(rows.len(), page_size);
        self.operations_ui.port_matrix_page =
            self.operations_ui.port_matrix_page.min(total_pages - 1);
        pagination_bar(
            ui,
            &mut self.operations_ui.port_matrix_page,
            total_pages,
            rows.len(),
        );
        ui.add_space(theme::SM);

        let start = self.operations_ui.port_matrix_page * page_size;
        render_port_table(self, ui, &rows, start, page_size);
        render_selected_port_summary(self, ui, &rows);
    }
}
