use super::resolution::view_for_resolution;
use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn action_queue_filter(&self) -> ReadinessActionFilter {
        ReadinessActionFilter::new(
            self.action_queue_severity_filter,
            self.action_queue_query.as_str(),
        )
        .with_resolution(self.action_queue_resolution_filter)
    }

    pub(in crate::app) fn filtered_readiness_actions(
        &self,
        diagnostics: &FleetDiagnostics,
    ) -> Vec<ReadinessAction> {
        filter_readiness_actions(diagnostics, &self.action_queue_filter())
    }

    pub(in crate::app) fn has_active_action_queue_filter(&self) -> bool {
        self.action_queue_severity_filter.is_some()
            || self.action_queue_resolution_filter.is_some()
            || !self.action_queue_query.trim().is_empty()
    }

    pub(in crate::app) fn clear_action_queue_filters(&mut self, diagnostics: &FleetDiagnostics) {
        self.action_queue_severity_filter = None;
        self.action_queue_resolution_filter = None;
        self.action_queue_query.clear();
        self.action_queue_page = 0;
        let actions = self.filtered_readiness_actions(diagnostics);
        self.ensure_visible_readiness_action_selection(&actions);
    }

    pub(in crate::app) fn focus_action_queue_severity(
        &mut self,
        diagnostics: &FleetDiagnostics,
        severity: CheckSeverity,
    ) {
        self.action_queue_severity_filter = Some(severity);
        self.action_queue_resolution_filter = None;
        self.action_queue_query.clear();
        self.action_queue_page = 0;
        let actions = self.filtered_readiness_actions(diagnostics);
        self.ensure_visible_readiness_action_selection(&actions);
    }

    pub(in crate::app) fn set_action_queue_resolution_filter(
        &mut self,
        diagnostics: &FleetDiagnostics,
        resolution: Option<DiagnosticResolution>,
    ) {
        self.action_queue_resolution_filter = resolution;
        self.action_queue_page = 0;
        let actions = self.filtered_readiness_actions(diagnostics);
        self.ensure_visible_readiness_action_selection(&actions);
    }

    pub(in crate::app) fn select_readiness_action(&mut self, action: &ReadinessAction) {
        self.selected_readiness_action = Some(action.key());
        self.selected_node = Some(action.node_id.clone());
    }

    pub(in crate::app) fn open_readiness_action_resolution(&mut self, action: &ReadinessAction) {
        self.select_readiness_action(action);
        self.selected_view = view_for_resolution(action.resolution);
        self.notice = Some(format!(
            "Opened {} for {}",
            action.resolution.label(),
            action.node_name
        ));
    }

    pub(in crate::app) fn selected_visible_readiness_action<'a>(
        &self,
        actions: &'a [ReadinessAction],
    ) -> Option<&'a ReadinessAction> {
        self.selected_readiness_action
            .as_ref()
            .and_then(|key| actions.iter().find(|action| key.matches(action)))
    }

    pub(in crate::app) fn ensure_visible_readiness_action_selection(
        &mut self,
        actions: &[ReadinessAction],
    ) {
        if actions.is_empty() {
            self.selected_readiness_action = None;
            return;
        }

        if self.selected_visible_readiness_action(actions).is_some() {
            return;
        }

        self.select_readiness_action(&actions[0]);
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
