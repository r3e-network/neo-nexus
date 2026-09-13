//! Settling the runtime-state rows a previous process left behind.
//!
//! Deliberately not a blanket "mark everything stopped": a workbench killed with
//! SIGKILL leaves its nodes running as orphans. Clearing those rows would lose
//! the only handle on them, and the next Start would launch a second node onto
//! the same ports. A node whose pid is answered by a *different* program is
//! settled too, but reported separately — the number was recycled, so the old
//! process is gone and something unrelated now holds its identity.

use crate::{
    events::{EventKind, EventSeverity, NewRuntimeEvent},
    supervisor::{recorded_process, RecordedProcess},
    types::NodeStatus,
};

use super::state::EngineState;

pub(super) fn reconcile_startup(state: &EngineState) {
    let mut settled = Vec::new();
    let mut recycled = Vec::new();
    for node in state.nodes() {
        if !matches!(node.status, NodeStatus::Running | NodeStatus::Starting) {
            continue;
        }
        // One classification per node: each probe reads the process table.
        let verdict = recorded_process(&node);
        if verdict == RecordedProcess::Alive {
            if node.status.is_starting() {
                // A controller can die after claiming a restart but before it
                // changes the still-live old process. Preserve that process and
                // settle the transient lease back to Running.
                let _ = state.repository.transition_node_status(
                    &node.id,
                    NodeStatus::Starting,
                    node.pid,
                    NodeStatus::Running,
                    node.pid,
                );
            }
            continue;
        }
        let _ = state.repository.transition_node_status(
            &node.id,
            node.status,
            node.pid,
            NodeStatus::Stopped,
            None,
        );
        match verdict {
            RecordedProcess::Reused => recycled.push(node.name),
            _ => settled.push(node.name),
        }
    }
    let total = settled.len() + recycled.len();
    if total == 0 {
        return;
    }
    let mut message = format!("Recovered {total} stale runtime state records");
    if !recycled.is_empty() {
        message.push_str(&format!(
            "; {} pid(s) now belong to another program",
            recycled.len()
        ));
    }
    let _ = state.repository.record_event(NewRuntimeEvent {
        node_id: None,
        node_name: None,
        kind: EventKind::RuntimeRecovered,
        severity: EventSeverity::Warning,
        message,
    });
}
