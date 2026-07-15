use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn schedule_watchdog_restart(&mut self, node: &NodeConfig, reason: &str) {
        let outcome = self.watchdog.record_failure(&node.id, Instant::now());
        if let Err(error) = self
            .repository
            .update_node_status(&node.id, NodeStatus::Error, None)
        {
            self.session.notice = Some(error.to_string());
            return;
        }

        self.session.notice = Some(match outcome {
            RestartOutcome::Disabled => {
                let message = format!("{reason}; watchdog disabled by policy");
                self.record_node_event(
                    node,
                    EventKind::WatchdogSkipped,
                    EventSeverity::Warning,
                    message.clone(),
                );
                message
            }
            RestartOutcome::Scheduled { attempt, delay } => {
                let message = format!(
                    "{}; watchdog restart attempt {} in {}",
                    reason,
                    attempt,
                    format_duration(delay)
                );
                self.record_node_event(
                    node,
                    EventKind::WatchdogScheduled,
                    EventSeverity::Warning,
                    message.clone(),
                );
                message
            }
            RestartOutcome::Exhausted { attempts } => {
                let message = format!("{}; watchdog exhausted after {} attempts", reason, attempts);
                self.record_node_event(
                    node,
                    EventKind::WatchdogExhausted,
                    EventSeverity::Critical,
                    message.clone(),
                );
                message
            }
        });
    }

    pub(in crate::app) fn schedule_sidecar_watchdog_restart(
        &mut self,
        process_id: &str,
        label: &str,
        reason: &str,
    ) {
        let outcome = self.watchdog.record_failure(process_id, Instant::now());
        self.session.notice = Some(match outcome {
            RestartOutcome::Disabled => {
                let message = format!("{reason}; watchdog disabled by policy");
                self.record_event(
                    None,
                    None,
                    EventKind::WatchdogSkipped,
                    EventSeverity::Warning,
                    message.clone(),
                );
                message
            }
            RestartOutcome::Scheduled { attempt, delay } => {
                let message = format!(
                    "{}; watchdog restart attempt {} in {}",
                    reason,
                    attempt,
                    format_duration(delay)
                );
                self.record_event(
                    None,
                    None,
                    EventKind::WatchdogScheduled,
                    EventSeverity::Warning,
                    message.clone(),
                );
                message
            }
            RestartOutcome::Exhausted { attempts } => {
                let message = format!("{reason}; watchdog exhausted after {attempts} attempts");
                self.record_event(
                    None,
                    None,
                    EventKind::WatchdogExhausted,
                    EventSeverity::Critical,
                    message.clone(),
                );
                message
            }
        });

        if let Some(report) = &self.private_network_sidecar_report {
            if !report
                .sidecars
                .iter()
                .any(|sidecar| sidecar.process.id == process_id)
            {
                self.session.notice = Some(format!(
                    "signer-sidecar:{label} exited but is no longer present in loaded sidecar specs"
                ));
            }
        }
    }
}
