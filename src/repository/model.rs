use std::path::PathBuf;

use crate::events::NewRuntimeEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreNodeOutcome {
    Created,
    Updated,
}

/// Launch material retained from a backup while the active node is deliberately
/// unbound. It can be exported again for fidelity, but it never reaches a
/// process command until an operator supplies a local replacement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QuarantinedRuntimeSpec {
    pub(crate) binary_path: PathBuf,
    pub(crate) args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSetting {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoredRuntimeEvent {
    pub occurred_at_unix: u64,
    pub event: NewRuntimeEvent,
}
