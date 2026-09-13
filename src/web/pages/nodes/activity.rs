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

    let encoded_name = html::urlencoding_lite(&node.name);
    format!(
        r#"<div class="panel" style="margin-top: 16px;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;">
                <div style="display: flex; align-items: center; gap: 8px;">
                    <span style="font-size: 18px;">📜</span>
                    <h3 style="margin: 0;">Instance Activity & Audit Trail</h3>
                </div>
                <a href="/events?q={encoded_name}" class="btn small">View Full Audit Journal →</a>
            </div>
            <p class="muted" style="font-size: 13px; margin-bottom: 12px;">
                Cryptographically tracked lifecycle operations, Hermes copilot interventions, and supervisor state transitions for this virtual instance.
            </p>
            {content}
        </div>"#,
        encoded_name = encoded_name,
        content = content,
    )
}
