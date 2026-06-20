use std::str::FromStr;

use anyhow::{Context, Result};

use crate::{
    events::{EventKind, EventSeverity, NewRuntimeEvent},
    repository::RestoredRuntimeEvent,
};

use super::super::schema::EventBackup;

pub(in crate::backup) fn restored_event(backup: &EventBackup) -> Result<RestoredRuntimeEvent> {
    validate_event_backup(backup)?;
    Ok(RestoredRuntimeEvent {
        occurred_at_unix: backup.occurred_at_unix,
        event: NewRuntimeEvent {
            node_id: backup.node_id.clone(),
            node_name: backup.node_name.clone(),
            kind: EventKind::from_str(&backup.kind)
                .with_context(|| format!("backup event {} has invalid kind", backup.id))?,
            severity: EventSeverity::from_str(&backup.severity)
                .with_context(|| format!("backup event {} has invalid severity", backup.id))?,
            message: backup.message.clone(),
        },
    })
}

pub(in crate::backup) fn validate_event_backup(event: &EventBackup) -> Result<()> {
    if event.id <= 0 {
        anyhow::bail!("backup event id must be greater than 0");
    }
    EventKind::from_str(&event.kind)
        .with_context(|| format!("backup event {} has invalid kind", event.id))?;
    EventSeverity::from_str(&event.severity)
        .with_context(|| format!("backup event {} has invalid severity", event.id))?;
    Ok(())
}
