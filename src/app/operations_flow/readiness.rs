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

    pub(in crate::app) fn has_active_readiness_check_filter(&self) -> bool {
        self.readiness_check_severity_filter.is_some()
            || !self.readiness_check_query.trim().is_empty()
    }

    pub(in crate::app) fn clear_readiness_check_filters(&mut self, node: &NodeDiagnostics) {
        self.readiness_check_severity_filter = None;
        self.readiness_check_query.clear();
        self.readiness_check_page = 0;
        let checks = self.filtered_readiness_checks(node);
        self.ensure_visible_readiness_check_selection(&checks);
    }

    pub(in crate::app) fn focus_readiness_check_severity(
        &mut self,
        node: &NodeDiagnostics,
        severity: CheckSeverity,
    ) {
        self.readiness_check_severity_filter = Some(severity);
        self.readiness_check_query.clear();
        self.readiness_check_page = 0;
        let checks = self.filtered_readiness_checks(node);
        self.ensure_visible_readiness_check_selection(&checks);
    }

    pub(in crate::app) fn select_readiness_check(&mut self, check: &DiagnosticCheck) {
        self.selected_readiness_check = Some(check.key());
    }

    pub(in crate::app) fn selected_visible_readiness_check<'a>(
        &self,
        checks: &'a [DiagnosticCheck],
    ) -> Option<&'a DiagnosticCheck> {
        self.selected_readiness_check
            .as_ref()
            .and_then(|key| checks.iter().find(|check| key.matches(check)))
    }

    pub(in crate::app) fn ensure_visible_readiness_check_selection(
        &mut self,
        checks: &[DiagnosticCheck],
    ) {
        if checks.is_empty() {
            self.selected_readiness_check = None;
            return;
        }

        if self.selected_visible_readiness_check(checks).is_some() {
            return;
        }

        self.select_readiness_check(&checks[0]);
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
