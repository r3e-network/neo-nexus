use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn drain_rpc_health_results(&mut self) {
        while let Ok(result) = self.rpc_health_results.try_recv() {
            self.rpc_health_pending.remove(&result.node.id);
            let Some(node) = self.fleet
                .nodes
                .iter()
                .find(|node| node.id == result.node.id)
                .cloned()
            else {
                continue;
            };
            if !node.status.is_running() {
                continue;
            }

            let previous_status = self
                .repository
                .latest_rpc_health(&node.id)
                .ok()
                .flatten()
                .map(|record| record.status);
            let message = rpc_health_notice(&result.report);
            if let Err(error) = self.repository.record_rpc_health(&node, &result.report) {
                self.session.notice = Some(error.to_string());
                continue;
            }
            if let Err(error) = self
                .repository
                .prune_rpc_health_keep_recent_per_node(RPC_HEALTH_RETAIN_PER_NODE)
            {
                self.session.notice = Some(format!("{message}; RPC health pruning failed: {error}"));
                continue;
            }

            if should_record_rpc_health_event(previous_status, result.report.status) {
                self.record_node_event(
                    &node,
                    EventKind::RpcHealthChecked,
                    rpc_health_event_severity(result.report.status),
                    format!("Automatic RPC health: {message}"),
                );
            }
        }
    }
}
