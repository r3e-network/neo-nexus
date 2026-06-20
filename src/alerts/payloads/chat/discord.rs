use serde_json::{json, Value};

use crate::{
    alerts::{text::truncate_for_message, MESSAGE_LIMIT},
    events::{EventSeverity, RuntimeEvent},
};

use super::super::common::alert_summary;

pub(in crate::alerts) fn discord_alert_payload(
    event: &RuntimeEvent,
    application_version: &str,
) -> Value {
    let node = event.node_name.as_deref().unwrap_or("workspace");
    json!({
        "username": "NeoNexus",
        "content": truncate_for_message(&alert_summary(event), MESSAGE_LIMIT),
        "embeds": [
            {
                "title": format!("NeoNexus {} alert", event.severity),
                "description": truncate_for_message(&event.message, MESSAGE_LIMIT),
                "color": discord_severity_color(event.severity),
                "fields": [
                    {
                        "name": "Kind",
                        "value": event.kind.to_string(),
                        "inline": true
                    },
                    {
                        "name": "Scope",
                        "value": node,
                        "inline": true
                    },
                    {
                        "name": "Event",
                        "value": format!("#{} @ {}", event.id, event.occurred_at_unix),
                        "inline": false
                    }
                ],
                "footer": {
                    "text": format!("NeoNexus {}", application_version)
                }
            }
        ],
    })
}

fn discord_severity_color(severity: EventSeverity) -> u32 {
    match severity {
        EventSeverity::Info => 0x2563eb,
        EventSeverity::Warning => 0xf59e0b,
        EventSeverity::Critical => 0xdc2626,
    }
}
