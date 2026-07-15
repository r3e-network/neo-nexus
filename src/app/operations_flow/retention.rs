use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn prune_event_journal(&mut self) {
        let keep_before_audit_event = EVENT_RETAIN_AFTER_PRUNE.saturating_sub(1);
        match self
            .repository
            .prune_events_keep_recent(keep_before_audit_event)
        {
            Ok(deleted) => {
                self.operations_ui.event_page = 0;
                self.operations_ui.selected_event = None;
                let message = format!(
                    "Event journal pruned: {deleted} removed, retaining {EVENT_RETAIN_AFTER_PRUNE}"
                );
                self.record_event_notice(EventKind::EventsPruned, EventSeverity::Warning, message);
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn prune_rpc_health_history(&mut self) {
        match self
            .repository
            .prune_rpc_health_keep_recent_per_node(RPC_HEALTH_RETAIN_PER_NODE)
        {
            Ok(deleted) => {
                let message = format!(
                    "RPC health history pruned: {deleted} removed, retaining {RPC_HEALTH_RETAIN_PER_NODE} per node"
                );
                self.record_event_notice(EventKind::EventsPruned, EventSeverity::Info, message);
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn prune_remote_federation_history(&mut self) {
        match self
            .repository
            .prune_remote_server_probes_keep_recent_per_profile(REMOTE_PROBE_RETAIN_PER_PROFILE)
        {
            Ok(deleted) => {
                let message = format!(
                    "Remote Federation history pruned: {deleted} removed, retaining {REMOTE_PROBE_RETAIN_PER_PROFILE} per profile"
                );
                self.record_event_notice(EventKind::EventsPruned, EventSeverity::Info, message);
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }
}
