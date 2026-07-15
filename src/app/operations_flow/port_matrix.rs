use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn port_matrix_filter(&self) -> PortMatrixFilter {
        PortMatrixFilter::new(
            self.operations_ui.port_matrix_status_filter,
            self.operations_ui.port_matrix_network_filter,
            self.operations_ui.port_matrix_health_filter,
            self.operations_ui.port_matrix_query.as_str(),
        )
    }

    pub(in crate::app) fn filtered_port_matrix_rows(
        &self,
        diagnostics: &FleetDiagnostics,
    ) -> Vec<PortMatrixRow> {
        filter_port_matrix_rows(&self.fleet.nodes, diagnostics, &self.port_matrix_filter())
    }

    pub(in crate::app) fn has_active_port_matrix_filter(&self) -> bool {
        self.operations_ui.port_matrix_status_filter.is_some()
            || self.operations_ui.port_matrix_network_filter.is_some()
            || self.operations_ui.port_matrix_health_filter.is_some()
            || !self.operations_ui.port_matrix_query.trim().is_empty()
    }

    pub(in crate::app) fn clear_port_matrix_filters(&mut self, diagnostics: &FleetDiagnostics) {
        self.operations_ui.port_matrix_status_filter = None;
        self.operations_ui.port_matrix_network_filter = None;
        self.operations_ui.port_matrix_health_filter = None;
        self.operations_ui.port_matrix_query.clear();
        self.operations_ui.port_matrix_page = 0;
        let rows = self.filtered_port_matrix_rows(diagnostics);
        self.ensure_visible_port_matrix_selection(&rows);
    }

    pub(in crate::app) fn focus_blocked_ports(&mut self, diagnostics: &FleetDiagnostics) {
        self.operations_ui.port_matrix_status_filter = None;
        self.operations_ui.port_matrix_network_filter = None;
        self.operations_ui.port_matrix_health_filter = Some(CheckSeverity::Critical);
        self.operations_ui.port_matrix_query.clear();
        self.operations_ui.port_matrix_page = 0;
        let rows = self.filtered_port_matrix_rows(diagnostics);
        self.ensure_visible_port_matrix_selection(&rows);
    }

    pub(in crate::app) fn select_port_matrix_row(&mut self, row: &PortMatrixRow) {
        self.fleet.selected_node = Some(row.node_id.clone());
    }

    pub(in crate::app) fn selected_visible_port_matrix_row<'a>(
        &self,
        rows: &'a [PortMatrixRow],
    ) -> Option<&'a PortMatrixRow> {
        self.fleet.selected_node
            .as_ref()
            .and_then(|node_id| rows.iter().find(|row| row.node_id == *node_id))
    }

    pub(in crate::app) fn ensure_visible_port_matrix_selection(&mut self, rows: &[PortMatrixRow]) {
        if rows.is_empty() {
            return;
        }

        if self.selected_visible_port_matrix_row(rows).is_some() {
            return;
        }

        self.select_port_matrix_row(&rows[0]);
    }

    pub(in crate::app) fn clamp_port_matrix_page(&mut self, diagnostics: &FleetDiagnostics) {
        let rows = self.filtered_port_matrix_rows(diagnostics);
        self.operations_ui.port_matrix_page =
            clamp_page(self.operations_ui.port_matrix_page, rows.len(), PORT_MATRIX_PAGE_SIZE);
    }
}
