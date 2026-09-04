use super::*;
use crate::watchdog::unix_millis;

impl LoopState {
    pub(super) fn initialize_recovery(&mut self, state: &EngineState) {
        match state
            .repository
            .recover_interrupted_node_attempts(unix_millis())
        {
            Ok(nodes) if !nodes.is_empty() => {
                let _ = state.repository.record_event(NewRuntimeEvent {
                    node_id: None, node_name: None,
                    kind: EventKind::RuntimeRecovered, severity: EventSeverity::Warning,
                    message: format!("Recovered {} interrupted node restart attempt(s); consumed retry budgets were retained", nodes.len()),
                });
            }
            Err(error) => self.recovery_problem(state, &error),
            _ => (),
        }
        self.refresh_recovery(state);
    }

    pub(super) fn refresh_recovery(&mut self, state: &EngineState) -> bool {
        let result = (|| -> anyhow::Result<_> {
            Ok((
                state.repository.load_watchdog_policy()?,
                state.repository.load_node_recoveries()?,
            ))
        })();
        match result {
            Ok((policy, records)) => {
                self.watchdog = Watchdog::restored(policy, &records, Instant::now(), unix_millis());
                true
            }
            Err(error) => {
                self.recovery_problem(state, &error);
                false
            }
        }
    }

    fn recovery_problem(&mut self, state: &EngineState, error: &anyhow::Error) {
        let message = format!("Node automatic recovery is blocked by its durable state: {error}");
        if self.recovery_error.as_deref() != Some(&message) {
            eprintln!("neo-nexus: {message}");
            let _ = state.repository.record_event(NewRuntimeEvent {
                node_id: None,
                node_name: None,
                kind: EventKind::WatchdogSkipped,
                severity: EventSeverity::Critical,
                message: message.clone(),
            });
            self.recovery_error = Some(message);
        }
    }

    pub(super) fn schedule_restart(
        &mut self,
        state: &EngineState,
        node: &NodeConfig,
        reason: &str,
    ) -> bool {
        match state
            .repository
            .schedule_node_recovery(node, NodeStatus::Crashed, unix_millis())
        {
            Ok(Some(outcome)) => {
                self.recovery_error = None;
                self.journal_restart(state, node, reason, outcome);
                self.refresh_recovery(state);
                true
            }
            Ok(None) => false,
            Err(error) => {
                self.recovery_problem(state, &error);
                false
            }
        }
    }

    pub(super) fn run_due_restarts(&mut self, state: &EngineState) {
        if !self.refresh_recovery(state) {
            return;
        }
        for due in self.watchdog.due_restarts(Instant::now()) {
            let claim = match state
                .repository
                .claim_node_recovery(&due.node_id, unix_millis())
            {
                Ok(Some(claim)) => {
                    self.recovery_error = None;
                    claim
                }
                Ok(None) => continue,
                Err(error) => {
                    self.recovery_problem(state, &error);
                    continue;
                }
            };
            let Some(node) = state
                .nodes()
                .into_iter()
                .find(|node| node.id == claim.node_id)
            else {
                continue;
            };
            let outcome = launch_node_guarded(state, &node, LaunchAction::Start, || {
                state.repository.validate_recovery_claim(&claim)
            });
            match outcome {
                Ok(message) => state.journal(
                    &node,
                    EventKind::WatchdogRestarted,
                    EventSeverity::Warning,
                    format!("watchdog attempt {}: {message}", claim.attempt),
                ),
                Err(error) => {
                    state.journal(
                        &node,
                        EventKind::NodeStartFailed,
                        EventSeverity::Critical,
                        format!("watchdog attempt {} failed: {error}", claim.attempt),
                    );
                    let supervisor = state.supervisor();
                    // A failed cleanup retains its Child handle. Never launch a
                    // replacement while that process remains managed.
                    if supervisor.is_managing(&node.id) {
                        continue;
                    }
                    match state.repository.fail_node_recovery(&claim, unix_millis()) {
                        Ok(Some(outcome)) => {
                            self.journal_restart(state, &node, "automatic launch failed", outcome)
                        }
                        Ok(None) => (),
                        Err(error) => self.recovery_problem(state, &error),
                    }
                }
            }
        }
        self.refresh_recovery(state);
    }
}
