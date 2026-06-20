use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn readiness_check_filter(&self) -> DiagnosticCheckFilter {
        DiagnosticCheckFilter::new(
            self.readiness_check_severity_filter,
            self.readiness_check_query.as_str(),
        )
    }

    pub(in crate::app) fn filtered_readiness_checks(
        &self,
        node: &NodeDiagnostics,
    ) -> Vec<DiagnosticCheck> {
        filter_diagnostic_checks(&node.checks, &self.readiness_check_filter())
    }

    pub(in crate::app) fn clamp_readiness_check_page(&mut self, node: &NodeDiagnostics) {
        let checks = self.filtered_readiness_checks(node);
        self.readiness_check_page = clamp_page(
            self.readiness_check_page,
            checks.len(),
            READINESS_CHECK_PAGE_SIZE,
        );
    }
}
