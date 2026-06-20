use std::collections::BTreeSet;

use anyhow::Result;

use super::uniqueness::insert_unique_value;
use crate::backup::{
    restore::validate_event_backup,
    schema::{EventBackup, WorkspaceBackup},
};

pub(super) fn validate_backup_events(backup: &WorkspaceBackup) -> Result<()> {
    let mut event_ids = BTreeSet::new();
    for event in &backup.events {
        validate_event_identity(event, &mut event_ids)?;
        validate_event_backup(event)?;
    }
    Ok(())
}

fn validate_event_identity(event: &EventBackup, event_ids: &mut BTreeSet<String>) -> Result<()> {
    if event.id > 0 {
        insert_unique_value(event_ids, "event id", &event.id.to_string())?;
    }
    Ok(())
}
