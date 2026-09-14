//! Cloud instance activity history and audit trail panel.

use crate::{
    core::node::NodeConfig,
    events::{EventSeverity, RuntimeEvent, RuntimeEventFilter},
    web::{html, time, WebState},
};

pub fn instance_activity_card(state: &WebState, node: &NodeConfig) -> String {
    let filter = RuntimeEventFilter::new(None, &node.name, 25);
    let events = state.workspace.list_events(filter).unwrap_or_default();
    let node_events: Vec<&RuntimeEvent> = events
        .iter()
        .filter(|e| {
            e.node_id.as_deref() == Some(node.id.as_str())
                || e.node_name.as_deref() == Some(node.name.as_str())
                || e.message.contains(&node.name)
                || e.message.contains(&node.id)
        })
        .take(8)
        .collect();

    let rows = node_events
        .iter()
        .map(|event| {
            let severity_badge = match event.severity {
                EventSeverity::Info => r#"<span class="badge running">INFO</span>"#,
                EventSeverity::Warning => r#"<span class="badge warn">WARN</span>"#,
                EventSeverity::Critical => r#"<span class="badge stopped">CRIT</span>"#,
            };
            html::row(&[
                html::raw_cell(&time::time_cell(Some(event.occurred_at_unix))),
                html::raw_cell(severity_badge),
                html::cell(event.kind.label()),
                html::cell(&event.message),
            ])
        })
        .collect::<Vec<_>>();

    let content = if rows.is_empty() {
        html::note("No lifecycle audit events recorded for this instance yet.")
    } else {
        html::table(&["Time", "Severity", "Action", "Audit Details"], &rows)
    };

    // Was `/events?q={name}` against a field named `query`, so the link
    // silently widened to the whole workspace journal. The journal takes a real
    // node scope now.
    let encoded_id = html::urlencoding_lite(&node.id);
    format!(
        r#"<div class="panel" style="margin-top: 16px;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;">
                <div style="display: flex; align-items: center; gap: 8px;">
                    <span style="font-size: 18px;">📜</span>
                    <h3 style="margin: 0;">Recent activity</h3>
                </div>
                <a href="/events?node={encoded_id}" class="btn small">Everything recorded for this node →</a>
            </div>
            <p class="muted" style="font-size: 13px; margin-bottom: 12px;">
                What this workspace recorded itself doing to this node. Nothing here is
                cryptographically tracked; it is a log this process writes.
            </p>
            {content}
        </div>"#,
        encoded_id = encoded_id,
        content = content,
    )
}
