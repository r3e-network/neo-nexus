//! Watching processes this server does not hold a handle for — started by the
//! CLI, or left alive across a restart — so their recorded status cannot stay
//! true after the process is gone.

use crate::{
    core::node::NodeConfig,
    events::EventKind,
    events::EventSeverity,
    supervisor::{recorded_process, RecordedProcess},
    types::NodeStatus,
};

use super::state::{EngineState, LoopState};

impl LoopState {
    pub(super) fn watch_external_processes(&mut self, state: &EngineState) {
        let supervisor = state.supervisor();
        let candidates: Vec<NodeConfig> = state
            .nodes()
            .into_iter()
            .filter(|node| node.status.is_running())
            .filter(|node| node.pid.is_some())
            .filter(|node| !supervisor.is_managing(&node.id))
            .collect();
        drop(supervisor);
        if candidates.is_empty() {
            return;
        }
        for node in candidates {
            let pid = node.pid.unwrap_or_default();
            let verdict = recorded_process(&node);
            if verdict == RecordedProcess::Alive {
                continue;
            }
            let settled = state.repository.transition_node_status(
                &node.id,
                node.status,
                node.pid,
                NodeStatus::Stopped,
                None,
            );
            if !matches!(settled, Ok(true)) {
                continue;
            }
            state.journal(
                &node,
                EventKind::NodeExited,
                EventSeverity::Warning,
                match verdict {
                    RecordedProcess::Gone => format!(
                        "{} is no longer running (pid {pid}); it was not supervised by this server",
                        node.name
                    ),
                    RecordedProcess::Reused => format!(
                        "{} stopped being tracked because pid {pid} now belongs to another executable",
                        node.name
                    ),
                    RecordedProcess::Alive => unreachable!(),
                },
            );
        }
    }
}
