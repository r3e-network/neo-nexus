use crate::{
    signer_client::{AuditRow, Caller, KeyPublic},
    web::{html, time},
};

pub fn audit_table(rows: &[AuditRow], keys: &[KeyPublic], callers: &[Caller]) -> String {
    if rows.is_empty() {
        return html::note("Nothing has been asked of this service yet.");
    }
    let body = rows
        .iter()
        .map(|row| {
            html::row(&[
                html::raw_cell(&time::time_cell(Some(row.recorded_at_unix))),
                html::cell(&row.action),
                html::raw_cell(&outcome_badge(&row.outcome)),
                html::cell(&name(keys, callers, row.key_id.as_deref(), true)),
                html::cell(&name(keys, callers, row.caller_id.as_deref(), false)),
                html::cell(row.origin.as_deref().unwrap_or("—")),
                html::cell(row.reason.as_deref().unwrap_or("")),
                html::cell(row.detail.as_deref().unwrap_or("")),
            ])
        })
        .collect::<Vec<_>>();
    html::table(
        &[
            "Time", "Event", "Result", "Key", "Caller", "Origin", "Code", "Detail",
        ],
        &body,
    )
}

fn name(keys: &[KeyPublic], callers: &[Caller], id: Option<&str>, is_key: bool) -> String {
    let Some(id) = id else {
        return "—".to_string();
    };
    let label = if is_key {
        keys.iter()
            .find(|key| key.key_id == id)
            .map(|key| key.label.clone())
    } else {
        callers
            .iter()
            .find(|caller| caller.id == id)
            .map(|caller| caller.label.clone())
    };
    label.map_or_else(|| id.to_string(), |label| format!("{label} ({id})"))
}

fn outcome_badge(outcome: &str) -> String {
    let class = match outcome {
        "allowed" => "running",
        "failed" => "error",
        _ => "stopped",
    };
    format!(
        r#"<span class="badge {class}">{}</span>"#,
        html::escape(outcome)
    )
}
