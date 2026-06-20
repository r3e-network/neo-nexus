use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn port_matrix_filter(&self) -> PortMatrixFilter {
        PortMatrixFilter::new(
            self.port_matrix_status_filter,
            self.port_matrix_network_filter,
            self.port_matrix_health_filter,
            self.port_matrix_query.as_str(),
        )
    }

    pub(in crate::app) fn filtered_port_matrix_rows(
        &self,
        diagnostics: &FleetDiagnostics,
    ) -> Vec<PortMatrixRow> {
        filter_port_matrix_rows(&self.nodes, diagnostics, &self.port_matrix_filter())
    }

    pub(in crate::app) fn clamp_port_matrix_page(&mut self, diagnostics: &FleetDiagnostics) {
        let rows = self.filtered_port_matrix_rows(diagnostics);
        self.port_matrix_page =
            clamp_page(self.port_matrix_page, rows.len(), PORT_MATRIX_PAGE_SIZE);
    }
}
