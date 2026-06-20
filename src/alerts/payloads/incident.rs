use serde_json::{json, Value};

use crate::{
    alerts::{text::truncate_for_message, MESSAGE_LIMIT, OPSGENIE_MESSAGE_LIMIT},
    events::{EventSeverity, RuntimeEvent},
};

use super::common::{alert_dedup_key, alert_summary};

pub(in crate::alerts) fn pagerduty_alert_payload(
    event: &RuntimeEvent,
    application_version: &str,
    routing_key: &str,
) -> Value {
    let node = event.node_name.as_deref().unwrap_or("workspace");
    let source = format!("NeoNexus/{node}");
    json!({
        "routing_key": routing_key,
        "event_action": "trigger",
        "dedup_key": alert_dedup_key(event),
        "payload": {
            "summary": truncate_for_message(&alert_summary(event), MESSAGE_LIMIT),
            "source": source,
            "severity": pagerduty_severity(event.severity),
            "component": event.kind.to_string(),
            "group": event.node_id.as_deref().unwrap_or("workspace"),
            "class": "NeoNexus",
            "custom_details": {
                "event_id": event.id,
                "occurred_at_unix": event.occurred_at_unix,
                "node_id": event.node_id.as_deref(),
                "node_name": event.node_name.as_deref(),
                "message": &event.message,
            }
        },
        "custom_details": {
            "application": "NeoNexus",
            "application_version": application_version,
            "event_kind": event.kind.to_string(),
        }
    })
}

pub(in crate::alerts) fn opsgenie_alert_payload(
    event: &RuntimeEvent,
    application_version: &str,
) -> Value {
    let node = event.node_name.as_deref().unwrap_or("workspace");
    let source = format!("NeoNexus/{node}");
    json!({
        "message": truncate_for_message(&alert_summary(event), OPSGENIE_MESSAGE_LIMIT),
        "alias": alert_dedup_key(event),
        "description": truncate_for_message(&event.message, MESSAGE_LIMIT),
        "source": source,
        "entity": event.node_id.as_deref().unwrap_or("workspace"),
        "priority": opsgenie_priority(event.severity),
        "tags": [
            "NeoNexus",
            event.severity.to_string(),
            event.kind.to_string(),
        ],
        "details": {
            "application": "NeoNexus",
            "application_version": application_version,
            "event_id": event.id.to_string(),
            "occurred_at_unix": event.occurred_at_unix.to_string(),
            "event_kind": event.kind.to_string(),
            "node_id": event.node_id.as_deref().unwrap_or(""),
            "node_name": event.node_name.as_deref().unwrap_or(""),
        }
    })
}

pub(in crate::alerts) fn datadog_event_payload(
    event: &RuntimeEvent,
    application_version: &str,
) -> Value {
    let node = event.node_name.as_deref().unwrap_or("workspace");
    json!({
        "data": {
            "type": "event",
            "attributes": {
                "aggregation_key": alert_dedup_key(event),
                "category": "alert",
                "host": node,
                "integration_id": "neonexus",
                "message": truncate_for_message(&event.message, MESSAGE_LIMIT),
                "tags": datadog_tags(event, application_version),
                "title": truncate_for_message(&alert_summary(event), MESSAGE_LIMIT),
                "attributes": {
                    "status": datadog_status(event.severity),
                    "priority": datadog_priority(event.severity),
                    "custom": {
                        "application": "NeoNexus",
                        "application_version": application_version,
                        "event_id": event.id,
                        "event_kind": event.kind.to_string(),
                        "node_id": event.node_id.as_deref().unwrap_or(""),
                        "node_name": event.node_name.as_deref().unwrap_or(""),
                        "occurred_at_unix": event.occurred_at_unix,
                    }
                }
            }
        }
    })
}

fn pagerduty_severity(severity: EventSeverity) -> &'static str {
    match severity {
        EventSeverity::Info => "info",
        EventSeverity::Warning => "warning",
        EventSeverity::Critical => "critical",
    }
}

fn opsgenie_priority(severity: EventSeverity) -> &'static str {
    match severity {
        EventSeverity::Info => "P5",
        EventSeverity::Warning => "P3",
        EventSeverity::Critical => "P1",
    }
}

fn datadog_status(severity: EventSeverity) -> &'static str {
    match severity {
        EventSeverity::Info => "info",
        EventSeverity::Warning => "warning",
        EventSeverity::Critical => "error",
    }
}

fn datadog_priority(severity: EventSeverity) -> &'static str {
    match severity {
        EventSeverity::Info => "3",
        EventSeverity::Warning => "2",
        EventSeverity::Critical => "1",
    }
}

fn datadog_tags(event: &RuntimeEvent, application_version: &str) -> Vec<String> {
    let mut tags = vec![
        "service:neo-nexus".to_string(),
        format!("version:{application_version}"),
        format!("severity:{}", event.severity),
        format!("event_kind:{}", event.kind),
    ];
    if let Some(node_id) = &event.node_id {
        tags.push(format!("node_id:{node_id}"));
    }
    if let Some(node_name) = &event.node_name {
        tags.push(format!("node_name:{node_name}"));
    }
    tags
}
