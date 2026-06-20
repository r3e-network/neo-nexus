use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn reconcile_missing_process_records(&mut self) {
        self.refresh_metrics_now();
        let missing_processes = self.metrics_snapshot.missing_processes.clone();
        if missing_processes.is_empty() {
            self.notice = Some("No missing running processes to reconcile".to_string());
            return;
        }

        let mut reconciled = 0usize;
        for missing in missing_processes {
            let Some(node) = self
                .nodes
                .iter()
                .find(|node| node.id == missing.node_id)
                .cloned()
            else {
                continue;
            };
            if node.status != NodeStatus::Running || node.pid != Some(missing.pid) {
                continue;
            }
            match self
                .repository
                .update_node_status(&node.id, NodeStatus::Stopped, None)
            {
                Ok(()) => {
                    self.watchdog.clear(&node.id);
                    reconciled += 1;
                    self.record_node_event(
                        &node,
                        EventKind::RuntimeStateReconciled,
                        EventSeverity::Warning,
                        format!(
                            "{} marked stopped because PID {} is no longer observable",
                            node.name, missing.pid
                        ),
                    );
                }
                Err(error) => {
                    self.notice = Some(error.to_string());
                    return;
                }
            }
        }

        self.reload_nodes();
        self.refresh_metrics_now();
        self.notice = Some(format!(
            "Runtime state reconciled: {reconciled} missing process record(s) stopped"
        ));
    }
}
