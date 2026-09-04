use super::*;

/// Recover routing conservatively: an unreadable cursor cannot authorize
/// skipping retained events. A replay can duplicate deliveries, but not lose them.
pub(super) fn alert_progress(state: &EngineState) -> (i64, BTreeMap<i64, usize>) {
    let load = || -> anyhow::Result<_> {
        let latest = state.repository.latest_event_id()?;
        let Some((cursor, attempts)) = state.repository.load_alert_progress()? else {
            return Ok((latest, BTreeMap::new()));
        };
        anyhow::ensure!(
            cursor >= 0
                && cursor <= latest
                && attempts.iter().all(|(id, count)| {
                    *id > cursor
                        && *id <= latest
                        && (1..ALERT_DELIVERY_MAX_ATTEMPTS).contains(count)
                }),
            "invalid alert delivery progress range"
        );
        Ok((cursor, attempts))
    };
    let progress = match load() {
        Ok(progress) => progress,
        Err(error) => {
            diagnostic(state, format!(
                "Alert delivery progress could not be recovered ({error}); replaying from the oldest retained event. Previously delivered alerts may be repeated."
            ));
            (0, BTreeMap::new())
        }
    };
    if let Err(error) = state
        .repository
        .save_alert_progress(progress.0, &progress.1)
    {
        diagnostic(state, format!(
            "Alert delivery recovery progress could not be persisted ({error}); delivery resumes in memory and may replay after a restart."
        ));
    }
    progress
}

fn diagnostic(state: &EngineState, message: String) {
    eprintln!("neo-nexus: {message}");
    let _ = state.repository.record_event(NewRuntimeEvent {
        node_id: None,
        node_name: None,
        kind: EventKind::RuntimeRecovered,
        severity: EventSeverity::Critical,
        message,
    });
}

impl LoopState {
    /// Runs before the server serves requests and after the alert cursor is
    /// loaded, so newly diagnosed crashes are eligible for notification.
    pub(super) fn reconcile_startup(&mut self, state: &EngineState) {
        let supervisor = state.supervisor();
        for node in state.nodes() {
            if !matches!(node.status, NodeStatus::Running | NodeStatus::Starting)
                || supervisor.is_managing(&node.id)
            {
                continue;
            }
            match recorded_process(&node) {
                RecordedProcess::Alive => (),
                RecordedProcess::Reused => {
                    // Keep the suspicious identity visible. Neither startup nor
                    // a future due-restart tick may signal or replace it.
                    self.watchdog.clear(&node.id);
                    if let Err(error) =
                        state
                            .repository
                            .update_node_status(&node.id, NodeStatus::Error, node.pid)
                    {
                        diagnostic(
                            state,
                            format!(
                                "Could not persist startup PID mismatch for {}: {error}",
                                node.name
                            ),
                        );
                        continue;
                    }
                    state.journal(&node, EventKind::RuntimeRecovered, EventSeverity::Critical,
                        format!("{} recorded PID {} belongs to another executable after server restart; no process was signalled and automatic restart is blocked", node.name, node.pid.unwrap_or_default()));
                }
                RecordedProcess::Gone => {
                    if let Err(error) =
                        state
                            .repository
                            .update_node_status(&node.id, NodeStatus::Crashed, None)
                    {
                        diagnostic(
                            state,
                            format!("Could not persist startup crash for {}: {error}", node.name),
                        );
                        continue;
                    }
                    let reason =
                        "recorded process missing after server restart; exit code unavailable";
                    state.journal(
                        &node,
                        EventKind::RuntimeRecovered,
                        EventSeverity::Critical,
                        format!("{} {reason}", node.name),
                    );
                    self.queue_restart(state, &node, reason);
                }
            }
        }
    }
}
