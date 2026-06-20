use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn reconcile_supervised_processes(&mut self) {
        match self.supervisor.reap_finished() {
            Ok(exits) if exits.is_empty() => {}
            Ok(exits) => {
                let mut node_updated = 0;
                let mut sidecar_updated = 0;
                let mut last_notice = None;

                for exit in exits {
                    if self
                        .private_network_sidecar_pids
                        .remove(&exit.process_id)
                        .is_some()
                        || exit.process_id.starts_with("signer:")
                    {
                        sidecar_updated += 1;
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
                                |code| { format!("exit code {code}") }
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
                            last_notice = Some(message);
                        } else {
                            self.schedule_sidecar_watchdog_restart(
                                &exit.process_id,
                                label,
                                &message,
                            );
                            last_notice = self.notice.clone();
                        }
                        continue;
                    }

                    let Some(node) = self
                        .nodes
                        .iter()
                        .find(|node| node.id == exit.node_id)
                        .cloned()
                    else {
                        continue;
                    };

                    node_updated += 1;
                    if exit.exit_code == Some(0) {
                        self.watchdog.clear(&node.id);
                        if let Err(error) = self.repository.update_node_status(
                            &exit.node_id,
                            NodeStatus::Stopped,
                            None,
                        ) {
                            self.notice = Some(error.to_string());
                            return;
                        }
                        let message = exit_notice(&node.name, &exit);
                        self.record_node_event(
                            &node,
                            EventKind::NodeExited,
                            EventSeverity::Info,
                            message.clone(),
                        );
                        last_notice = Some(message);
                    } else {
                        let reason = self.exit_notice_with_log_diagnosis(&node, &exit);
                        self.schedule_watchdog_restart(&node, &reason);
                        last_notice = self.notice.clone();
                    }
                }

                if node_updated > 0 || sidecar_updated > 0 {
                    self.notice = last_notice;
                }
                if node_updated > 0 {
                    self.reload_nodes();
                }
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }
}
