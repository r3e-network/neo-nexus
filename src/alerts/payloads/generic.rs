use serde::Serialize;
use serde_json::{json, Value};

use crate::events::RuntimeEvent;

#[derive(Debug, Clone, Serialize)]
struct AlertWebhookEvent<'a> {
    id: i64,
    occurred_at_unix: u64,
    kind: String,
    severity: String,
    node_id: Option<&'a str>,
    node_name: Option<&'a str>,
    message: &'a str,
}

pub(super) fn generic_alert_payload(event: &RuntimeEvent, application_version: &str) -> Value {
    json!({
        "schema_version": 1,
        "application": "NeoNexus",
        "application_version": application_version,
        "event": AlertWebhookEvent {
            id: event.id,
            occurred_at_unix: event.occurred_at_unix,
            kind: event.kind.to_string(),
            severity: event.severity.to_string(),
            node_id: event.node_id.as_deref(),
            node_name: event.node_name.as_deref(),
            message: &event.message,
        },
    })
}
