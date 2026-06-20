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
            Err(error) => self.notice = Some(error.to_string()),
        }
    }
}
