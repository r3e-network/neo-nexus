use super::*;

impl NeoNexusApp {
    pub(super) fn record_node_event(
        &mut self,
        node: &NodeConfig,
        kind: EventKind,
        severity: EventSeverity,
        message: String,
    ) {
        self.record_event(
            Some(node.id.clone()),
            Some(node.name.clone()),
            kind,
            severity,
            message,
        );
    }

    /// Records a workspace-scoped (non-node) runtime event and surfaces the
    /// same message on the operator notice line. This is the global analog of
    /// [`Self::record_node_event`] and the most common pattern across the app
    /// flows, so it lives here to keep the event kind, severity, and notice in
    /// lockstep at every call site.
    pub(super) fn record_event_notice(
        &mut self,
        kind: EventKind,
        severity: EventSeverity,
        message: String,
    ) {
        self.record_event(None, None, kind, severity, message.clone());
        self.session.notice = Some(message);
    }

    pub(super) fn record_event(
        &mut self,
        node_id: Option<String>,
        node_name: Option<String>,
        kind: EventKind,
        severity: EventSeverity,
        message: String,
    ) {
        match self.repository.record_event(NewRuntimeEvent {
            node_id,
            node_name,
            kind,
            severity,
            message,
        }) {
            Ok(event) => self.route_alert_for_event(event),
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }
}
