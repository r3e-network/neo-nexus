//! High-level workspace data queries for frontends. Frontends read workspace
//! collections (snapshots, event journal) through these operations instead of
//! reaching into the repository's row API, so a view does not query SQLite
//! directly during paint and the persistence layer stays behind the core facade.

use anyhow::Result;

use crate::{events::RuntimeEvent, repository::Repository, snapshots::FastSyncSnapshot};

use super::operations::RuntimeEventFilter;

/// All registered fast-sync snapshots. The snapshot view loads the whole list
/// (it is small) and filters/verifies in memory, so this is a single read.
pub fn list_workspace_snapshots(repository: &Repository) -> Result<Vec<FastSyncSnapshot>> {
    repository.list_fast_sync_snapshots()
}

/// A page of the runtime event journal matching `filter`, newest first.
pub fn list_workspace_events(
    repository: &Repository,
    filter: RuntimeEventFilter,
) -> Result<Vec<RuntimeEvent>> {
    repository.list_events(filter)
}

/// How many events match `filter` — the journal's "matching" count.
pub fn count_workspace_events(
    repository: &Repository,
    filter: &RuntimeEventFilter,
) -> Result<usize> {
    repository.count_events(filter)
}
