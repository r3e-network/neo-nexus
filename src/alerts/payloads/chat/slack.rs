use serde_json::{json, Value};

use crate::{
    alerts::{text::truncate_for_message, MESSAGE_LIMIT},
    events::RuntimeEvent,
};

use super::super::common::alert_summary;

pub(in crate::alerts) fn slack_alert_payload(
    event: &RuntimeEvent,
    application_version: &str,
) -> Value {
    let summary = alert_summary(event);
    let node = event.node_name.as_deref().unwrap_or("workspace");
    json!({
        "text": summary,
        "blocks": [
            {
                "type": "header",
                "text": {
                    "type": "plain_text",
                    "text": format!("NeoNexus {} alert", event.severity),
                }
            },
            {
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": truncate_for_message(
                        &format!(
                            "*{}* on *{}*\n{}",
                            event.kind,
                            node,
                            event.message
                        ),
                        MESSAGE_LIMIT,
                    ),
                }
            },
            {
                "type": "context",
                "elements": [
                    {
                        "type": "mrkdwn",
                        "text": format!(
                            "Event #{} | {} | NeoNexus {}",
                            event.id,
                            event.occurred_at_unix,
                            application_version
                        )
                    }
                ]
            }
        ],
    })
}
