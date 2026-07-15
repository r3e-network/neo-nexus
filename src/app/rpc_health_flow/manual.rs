use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn check_selected_rpc_health(&mut self) {
        let Some(node) = self.selected_node().cloned() else {
            self.session.notice = Some("Select a node before checking RPC health".to_string());
            return;
        };

        let report = probe_node_rpc(&node, RPC_HEALTH_TIMEOUT);
        let message = rpc_health_notice(&report);
        if let Err(error) = self.repository.record_rpc_health(&node, &report) {
            self.session.notice = Some(error.to_string());
            return;
        }
        let pruned = match self
            .repository
            .prune_rpc_health_keep_recent_per_node(RPC_HEALTH_RETAIN_PER_NODE)
        {
            Ok(pruned) => pruned,
            Err(error) => {
                self.session.notice = Some(format!("{message}; RPC health pruning failed: {error}"));
                return;
            }
        };
        let event_message = if pruned == 0 {
            message.clone()
        } else {
            format!("{message}; pruned {pruned} old RPC health samples")
        };
        self.record_node_event(
            &node,
            EventKind::RpcHealthChecked,
            rpc_health_event_severity(report.status),
            event_message.clone(),
        );
        self.session.notice = Some(event_message);
    }
}
