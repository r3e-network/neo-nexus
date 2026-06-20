use serde_json::{json, Value};

use crate::{
    alerts::{text::truncate_for_message, MESSAGE_LIMIT},
    events::RuntimeEvent,
};

pub(in crate::alerts) fn telegram_alert_payload(
    event: &RuntimeEvent,
    application_version: &str,
    chat_id: &str,
) -> Value {
    let node = event.node_name.as_deref().unwrap_or("workspace");
    let text = format!(
        "<b>NeoNexus {} alert</b>\n<code>{}</code> on <b>{}</b>\n{}\n\nEvent #{} | {} | NeoNexus {}",
        escape_telegram_html(&event.severity.to_string()),
        escape_telegram_html(&event.kind.to_string()),
        escape_telegram_html(node),
        escape_telegram_html(&event.message),
        event.id,
        event.occurred_at_unix,
        escape_telegram_html(application_version),
    );

    json!({
        "chat_id": chat_id,
        "text": truncate_for_message(&text, MESSAGE_LIMIT),
        "parse_mode": "HTML",
    })
}

fn escape_telegram_html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            _ => escaped.push(character),
        }
    }
    escaped
}
