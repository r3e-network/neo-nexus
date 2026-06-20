use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn run_due_watchdog_restarts(&mut self) {
        let restarts = self.watchdog.due_restarts(Instant::now());
        for restart in restarts {
            if restart.node_id.starts_with("signer:") {
                self.restart_private_network_sidecar_by_id(&restart.node_id, restart.attempt);
                continue;
            }

            let Some(index) = self
                .nodes
                .iter()
                .position(|node| node.id == restart.node_id)
            else {
                self.watchdog.clear(&restart.node_id);
                continue;
            };

            if self.nodes[index].status == NodeStatus::Running {
                continue;
            }

            self.start_node_with_mode(
                index,
                StartMode::Watchdog {
                    attempt: restart.attempt,
                },
            );
        }
    }
}
