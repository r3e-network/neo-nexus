use anyhow::Result;

use crate::{
    backup::{restore::restored_event, schema::WorkspaceBackup},
    repository::Repository,
};

pub(super) fn restore_events(repository: &Repository, backup: &WorkspaceBackup) -> Result<usize> {
    let restored_events = backup
        .events
        .iter()
        .map(restored_event)
        .collect::<Result<Vec<_>>>()?;
    repository.restore_runtime_events(&restored_events)
}
