use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn action_queue_filter(&self) -> ReadinessActionFilter {
        ReadinessActionFilter::new(
            self.action_queue_severity_filter,
            self.action_queue_query.as_str(),
        )
    }

    pub(in crate::app) fn filtered_readiness_actions(
        &self,
        diagnostics: &FleetDiagnostics,
    ) -> Vec<ReadinessAction> {
        filter_readiness_actions(diagnostics, &self.action_queue_filter())
    }

    pub(in crate::app) fn clamp_action_queue_page(&mut self, diagnostics: &FleetDiagnostics) {
        let actions = self.filtered_readiness_actions(diagnostics);
        self.action_queue_page = clamp_page(
            self.action_queue_page,
            actions.len(),
            ACTION_QUEUE_PAGE_SIZE,
        );
    }
}
