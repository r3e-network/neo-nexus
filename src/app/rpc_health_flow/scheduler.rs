use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn schedule_due_rpc_health_checks(&mut self) {
        self.prune_deleted_rpc_health_runtime_state();
        if !self.rpc_health_monitor_policy.enabled {
            return;
        }

        let now = Instant::now();
        let interval = self.rpc_health_monitor_policy.interval_duration();
        let Some(node) = self.nodes.iter().find_map(|node| {
            if node.status != NodeStatus::Running || self.rpc_health_pending.contains(&node.id) {
                return None;
            }
            let due = self
                .rpc_health_last_started
                .get(&node.id)
                .is_none_or(|last_started| now.duration_since(*last_started) >= interval);
            due.then(|| node.clone())
        }) else {
            return;
        };

        self.rpc_health_pending.insert(node.id.clone());
        self.rpc_health_last_started.insert(node.id.clone(), now);
        let sender = self.rpc_health_sender.clone();
        let thread_node = node.clone();
        if let Err(error) = thread::Builder::new()
            .name(format!("neonexus-rpc-health-{}", thread_node.id))
            .spawn(move || {
                let report = probe_node_rpc(&thread_node, RPC_HEALTH_TIMEOUT);
                let _ = sender.send(RpcHealthProbeResult {
                    node: thread_node,
                    report,
                });
            })
        {
            self.rpc_health_pending.remove(&node.id);
            self.notice = Some(format!(
                "Unable to start RPC health probe for {}: {error}",
                node.name
            ));
        }
    }

    fn prune_deleted_rpc_health_runtime_state(&mut self) {
        let live_node_ids = self
            .nodes
            .iter()
            .map(|node| node.id.clone())
            .collect::<BTreeSet<_>>();
        self.rpc_health_pending
            .retain(|node_id| live_node_ids.contains(node_id));
        self.rpc_health_last_started
            .retain(|node_id, _| live_node_ids.contains(node_id));
    }
}
