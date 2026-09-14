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
/// The launch material a restored node arrived with, held back until an
/// operator confirms it.
///
/// A restore writes an empty binary path and stashes the real one here, so a
/// backup cannot make this host execute a path chosen on another. The operator
/// then has to retype argv the database is already holding — because nothing
/// showed it to them: this was `pub(crate)`, absent from `WorkspaceQueries`,
/// and read only by the backup exporter.
pub struct QuarantinedRuntimeSpec {
    pub binary_path: PathBuf,
    pub args: Vec<String>,
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
