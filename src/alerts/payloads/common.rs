use crate::events::RuntimeEvent;

use crate::alerts::{text::truncate_for_message, MESSAGE_LIMIT};

pub(super) fn alert_summary(event: &RuntimeEvent) -> String {
    let node = event.node_name.as_deref().unwrap_or("workspace");
    truncate_for_message(
        &format!(
            "[{}] {} on {}: {}",
            event.severity, event.kind, node, event.message
        ),
        MESSAGE_LIMIT,
    )
}

pub(super) fn alert_dedup_key(event: &RuntimeEvent) -> String {
    format!(
        "neonexus:{}:{}",
        event.kind,
        event.node_id.as_deref().unwrap_or("workspace")
    )
}
