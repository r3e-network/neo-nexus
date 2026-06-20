use crate::{events::RuntimeEvent, redaction::redact_sensitive_text};

use super::super::schema::EventBackup;

pub(in crate::backup) fn event_backup(event: RuntimeEvent) -> EventBackup {
    EventBackup {
        id: event.id,
        occurred_at_unix: event.occurred_at_unix,
        node_id: event.node_id,
        node_name: event.node_name,
        kind: event.kind.to_string(),
        severity: event.severity.to_string(),
        message: redact_sensitive_text(&event.message),
    }
}
