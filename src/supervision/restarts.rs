//! Deciding what a finished process means, and bringing a crashed node back
//! within the restart policy the Settings page describes.

use std::path::PathBuf;

use log::warn;

use crate::{
    core::{lifecycle::LaunchAction, node::NodeConfig},
    events::{EventKind, EventSeverity},
    health_events::{exit_notice, exit_was_clean},
    logs::LogReader,
    supervisor::{log_path_for, ProcessExit},
    types::NodeStatus,
    watchdog::RestartOutcome,
};

use super::{
    launch::launch_node,
    state::{EngineState, LoopState},
};

/// How much of a node's log is read to explain a crash. Enough for the tail
/// lines that carry the diagnosis, small enough to stay cheap on a tick.
const LOG_MAX_BYTES: usize = 64 * 1024;

impl LoopState {
    pub(super) fn sync_policy(&mut self, state: &EngineState) {
        let Ok(policy) = state.repository.load_watchdog_policy() else {
            return;
        };
        // `update_policy` clears pending restarts, so pushing it on every tick
        // would wipe a scheduled retry before its delay ever elapsed.
        if policy != self.applied_policy {
            self.watchdog.update_policy(policy);
            self.applied_policy = policy;
        }
    }

    /// Take every process the supervisor was watching that has now finished,
    /// and decide what it means.
    pub(super) fn reconcile_exits(&mut self, state: &EngineState) {
        let exits = match state.supervisor().reap_finished() {
            Ok(exits) => exits,
            Err(error) => {
                warn!("neo-nexus: failed to reap finished processes: {error}");
                return;
            }
        };
        if exits.is_empty() {
            return;
        }
        let nodes = state.nodes();
        for exit in exits {
            let Some(node) = nodes.iter().find(|node| node.id == exit.node_id) else {
                continue;
            };
            // `Stopped` is also the durable stop intent used by another CLI
            // process. A forced/TERM exit caused by that request is not a crash
            // and must never be scheduled for automatic restart.
            if !node.status.is_active() {
                self.watchdog.clear(&node.id);
                if node.status == NodeStatus::Stopped && node.pid == Some(exit.pid) {
                    let _ = state.repository.transition_node_status(
                        &node.id,
                        NodeStatus::Stopped,
                        Some(exit.pid),
                        NodeStatus::Stopped,
                        None,
                    );
                }
                continue;
            }
            if exit_was_clean(&exit) {
                self.watchdog.clear(&node.id);
                let _ = state.repository.transition_node_status(
                    &node.id,
                    node.status,
                    node.pid,
                    NodeStatus::Stopped,
                    None,
                );
                state.journal(
                    node,
                    EventKind::NodeExited,
                    EventSeverity::Info,
                    exit_notice(&node.name, &exit),
                );
                continue;
            }
            let reason = self.exit_notice_with_log(node, &exit, &state.workspace_child_dir("logs"));
            self.schedule_restart(state, node, &reason);
        }
    }

    /// A crash message is worth more with the log's own diagnosis attached: the
    /// exit code says it failed, the log says why.
    fn exit_notice_with_log(
        &self,
        node: &NodeConfig,
        exit: &ProcessExit,
        log_dir: &PathBuf,
    ) -> String {
        let base = exit_notice(&node.name, exit);
        let Ok(snapshot) = LogReader::snapshot(log_path_for(log_dir, node), LOG_MAX_BYTES) else {
            return base;
        };
        let diagnosis = LogReader::diagnose(&snapshot);
        match diagnosis.status {
            crate::logs::LogDiagnosisStatus::Critical
            | crate::logs::LogDiagnosisStatus::Warning => {
                format!("{base}; log diagnosis: {}", diagnosis.summary)
            }
            _ => base,
        }
    }

    pub(super) fn schedule_restart(
        &mut self,
        state: &EngineState,
        node: &NodeConfig,
        reason: &str,
    ) {
        let claimed = state.repository.transition_node_status(
            &node.id,
            node.status,
            node.pid,
            NodeStatus::Error,
            None,
        );
        if !matches!(claimed, Ok(true)) {
            // A concurrent Stop/Edit/Delete won. Its persisted decision takes
            // precedence over a stale exit snapshot.
            self.watchdog.clear(&node.id);
            return;
        }
        match self
            .watchdog
            .record_failure(&node.id, std::time::Instant::now())
        {
            RestartOutcome::Scheduled { attempt, delay } => state.journal(
                node,
                EventKind::WatchdogScheduled,
                EventSeverity::Warning,
                format!(
                    "{reason}; watchdog will retry in {}s (attempt {attempt})",
                    delay.as_secs()
                ),
            ),
            RestartOutcome::Exhausted { attempts } => state.journal(
                node,
                EventKind::WatchdogExhausted,
                EventSeverity::Critical,
                format!("{reason}; watchdog gave up after {attempts} attempts"),
            ),
            RestartOutcome::Disabled => state.journal(
                node,
                EventKind::NodeExited,
                EventSeverity::Warning,
                format!("{reason}; automatic restart is off"),
            ),
        }
    }

    pub(super) fn run_due_restarts(&mut self, state: &EngineState) {
        let due = self.watchdog.due_restarts(std::time::Instant::now());
        if due.is_empty() {
            return;
        }
        let nodes = state.nodes();
        for attempt in due {
            let Some(node) = nodes.iter().find(|node| node.id == attempt.node_id) else {
                self.watchdog.clear(&attempt.node_id);
                continue;
            };
            if node.status != NodeStatus::Error || node.pid.is_some() {
                // Most importantly, a CLI stop changes this to Stopped while a
                // retry is pending. Clear the stale schedule instead of undoing
                // the operator's request.
                self.watchdog.clear(&attempt.node_id);
                continue;
            }
            match launch_node(state, node, LaunchAction::Start) {
                Ok(message) => {
                    // A successful recovery completes this failure episode.
                    // A later, unrelated crash starts again at attempt one.
                    self.watchdog.clear(&node.id);
                    state.journal(
                        node,
                        EventKind::WatchdogRestarted,
                        EventSeverity::Warning,
                        format!("watchdog attempt {}: {message}", attempt.attempt),
                    );
                }
                Err(error) => {
                    let failure = format!("watchdog attempt {} failed: {error}", attempt.attempt);
                    state.journal(
                        node,
                        EventKind::NodeStartFailed,
                        EventSeverity::Critical,
                        failure.clone(),
                    );
                    // `due_restarts` consumes the pending timestamp. Without a
                    // fresh failure record, max_restart_attempts was effectively
                    // always one no matter what Settings said.
                    self.schedule_restart(state, node, &failure);
                }
            }
        }
    }
}
