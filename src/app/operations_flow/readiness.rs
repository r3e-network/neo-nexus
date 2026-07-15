use super::resolution::view_for_resolution;
use super::*;
use crate::app::domain::{diagnostic_check_resolution_counts, diagnostic_check_severity_counts};

impl NeoNexusApp {
    pub(in crate::app) fn readiness_check_filter(&self) -> DiagnosticCheckFilter {
        DiagnosticCheckFilter::new(
            self.operations_ui.readiness_check_severity_filter,
            self.operations_ui.readiness_check_query.as_str(),
        )
        .with_resolution(self.operations_ui.readiness_check_resolution_filter)
    }

    pub(in crate::app) fn filtered_readiness_checks(
        &self,
        node: &NodeDiagnostics,
    ) -> Vec<DiagnosticCheck> {
        filter_diagnostic_checks(&node.checks, &self.readiness_check_filter())
    }

    pub(in crate::app) fn readiness_check_resolution_counts(
        &self,
        node: &NodeDiagnostics,
    ) -> Vec<(DiagnosticResolution, usize)> {
        diagnostic_check_resolution_counts(&node.checks, &self.readiness_check_filter())
    }

    pub(in crate::app) fn readiness_check_severity_counts(
        &self,
        node: &NodeDiagnostics,
    ) -> Vec<(CheckSeverity, usize)> {
        diagnostic_check_severity_counts(&node.checks, &self.readiness_check_filter())
    }

    pub(in crate::app) fn has_active_readiness_check_filter(&self) -> bool {
        self.operations_ui.readiness_check_severity_filter.is_some()
            || self.operations_ui.readiness_check_resolution_filter.is_some()
            || !self.operations_ui.readiness_check_query.trim().is_empty()
    }

    pub(in crate::app) fn clear_readiness_check_filters(&mut self, node: &NodeDiagnostics) {
        self.operations_ui.readiness_check_severity_filter = None;
        self.operations_ui.readiness_check_resolution_filter = None;
        self.operations_ui.readiness_check_query.clear();
        self.operations_ui.readiness_check_page = 0;
        let checks = self.filtered_readiness_checks(node);
        self.ensure_visible_readiness_check_selection(&checks);
    }

    pub(in crate::app) fn focus_readiness_check_severity(
        &mut self,
        node: &NodeDiagnostics,
        severity: CheckSeverity,
    ) {
        self.operations_ui.readiness_check_severity_filter = Some(severity);
        self.operations_ui.readiness_check_resolution_filter = None;
        self.operations_ui.readiness_check_query.clear();
        self.operations_ui.readiness_check_page = 0;
        let checks = self.filtered_readiness_checks(node);
        self.ensure_visible_readiness_check_selection(&checks);
    }

    pub(in crate::app) fn set_readiness_check_resolution_filter(
        &mut self,
        node: &NodeDiagnostics,
        resolution: Option<DiagnosticResolution>,
    ) {
        self.operations_ui.readiness_check_resolution_filter = resolution;
        self.operations_ui.readiness_check_page = 0;
        let checks = self.filtered_readiness_checks(node);
        self.ensure_visible_readiness_check_selection(&checks);
    }

    pub(in crate::app) fn select_readiness_check(&mut self, check: &DiagnosticCheck) {
        self.operations_ui.selected_readiness_check = Some(check.key());
    }

    pub(in crate::app) fn open_readiness_check_resolution(
        &mut self,
        node: &NodeDiagnostics,
        check: &DiagnosticCheck,
    ) {
        self.fleet.selected_node = Some(node.node_id.clone());
        self.select_readiness_check(check);
        self.session.selected_view = view_for_resolution(check.resolution);
        self.session.notice = Some(format!(
            "Opened {} for {}",
            check.resolution.label(),
            node.node_name
        ));
    }

    pub(in crate::app) fn selected_visible_readiness_check<'a>(
        &self,
        checks: &'a [DiagnosticCheck],
    ) -> Option<&'a DiagnosticCheck> {
        self.operations_ui.selected_readiness_check
            .as_ref()
            .and_then(|key| checks.iter().find(|check| key.matches(check)))
    }

    pub(in crate::app) fn ensure_visible_readiness_check_selection(
        &mut self,
        checks: &[DiagnosticCheck],
    ) {
        if checks.is_empty() {
            self.operations_ui.selected_readiness_check = None;
            return;
        }

        if self.selected_visible_readiness_check(checks).is_some() {
            return;
        }

        self.select_readiness_check(&checks[0]);
    }

    pub(in crate::app) fn clamp_readiness_check_page(&mut self, node: &NodeDiagnostics) {
        let checks = self.filtered_readiness_checks(node);
        self.operations_ui.readiness_check_page = clamp_page(
            self.operations_ui.readiness_check_page,
            checks.len(),
            READINESS_CHECK_PAGE_SIZE,
        );
    }
}
