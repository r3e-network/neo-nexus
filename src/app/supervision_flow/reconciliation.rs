use super::*;

/// Result of reconciling a single finished process, used by
/// [`NeoNexusApp::reconcile_supervised_processes`] to drive counters and the
/// operator notice without leaking per-exit branching into the loop body.
enum ReapOutcome {
    /// The exit referenced a node that is no longer in the inventory.
    Skipped,
    /// A persistence error occurred; stop reconciling further exits.
    Aborted,
    /// A signer sidecar exit was handled, carrying its operator notice.
    Sidecar(Option<String>),
    /// A managed node exit was handled, carrying its operator notice.
    Node(Option<String>),
}

impl NeoNexusApp {
    pub(in crate::app) fn reconcile_supervised_processes(&mut self) {
        let exits = match self.supervisor.reap_finished() {
            Ok(exits) => exits,
            Err(error) => {
                self.session.notice = Some(error.to_string());
                return;
            }
        };
        if exits.is_empty() {
            return;
        }

        let mut node_updated = 0;
        let mut sidecar_updated = 0;
        let mut last_notice = None;
        for exit in exits {
            match self.reconcile_finished_process(&exit) {
                ReapOutcome::Skipped => continue,
                ReapOutcome::Aborted => return,
                ReapOutcome::Sidecar(notice) => {
                    sidecar_updated += 1;
                    last_notice = notice;
                }
                ReapOutcome::Node(notice) => {
                    node_updated += 1;
                    last_notice = notice;
                }
            }
        }

        if node_updated > 0 || sidecar_updated > 0 {
            self.session.notice = last_notice;
        }
        if node_updated > 0 {
            self.reload_nodes();
        }
    }

    /// Reconciles a single finished process: classifies it as a signer sidecar
    /// or a managed node exit, records the audit event, and either clears the
    /// watchdog (clean exit) or schedules a restart (abnormal exit).
    fn reconcile_finished_process(&mut self, exit: &ProcessExit) -> ReapOutcome {
        if self
            .private_network_sidecar_pids
            .remove(&exit.process_id)
            .is_some()
            || exit.process_id.starts_with("signer:")
        {
            return self.reconcile_sidecar_exit(exit);
        }

        let Some(node) = self.fleet
            .nodes
            .iter()
            .find(|node| node.id == exit.node_id)
            .cloned()
        else {
            return ReapOutcome::Skipped;
        };

        if exit.exit_code == Some(0) {
            self.watchdog.clear(&node.id);
            if let Err(error) =
                self.repository
                    .update_node_status(&exit.node_id, NodeStatus::Stopped, None)
            {
                self.session.notice = Some(error.to_string());
                return ReapOutcome::Aborted;
            }
            let message = exit_notice(&node.name, exit);
            self.record_node_event(
                &node,
                EventKind::NodeExited,
                EventSeverity::Info,
                message.clone(),
            );
            ReapOutcome::Node(Some(message))
        } else {
            let reason = self.exit_notice_with_log_diagnosis(&node, exit);
            self.schedule_watchdog_restart(&node, &reason);
            ReapOutcome::Node(self.session.notice.clone())
        }
    }

    /// Handles a finished signer sidecar process, recording the exit event and
    /// either clearing its watchdog (clean exit) or scheduling a restart.
    fn reconcile_sidecar_exit(&mut self, exit: &ProcessExit) -> ReapOutcome {
        let severity = if exit.exit_code == Some(0) {
            EventSeverity::Info
        } else {
            EventSeverity::Warning
        };
        let label = exit
            .process_id
            .strip_prefix("signer:")
            .unwrap_or(exit.process_id.as_str());
        let message = format!(
            "signer-sidecar:{} exited with {}",
            label,
            exit.exit_code.map_or_else(
                || "no exit code".to_string(),
                |code| format!("exit code {code}")
            )
        );
        self.record_event(
            None,
            None,
            EventKind::PrivateNetworkSignerSidecarExited,
            severity,
            message.clone(),
        );
        if exit.exit_code == Some(0) {
            self.watchdog.clear(&exit.process_id);
            ReapOutcome::Sidecar(Some(message))
        } else {
            self.schedule_sidecar_watchdog_restart(&exit.process_id, label, &message);
            ReapOutcome::Sidecar(self.session.notice.clone())
        }
    }
}
