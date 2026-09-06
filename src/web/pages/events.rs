//! Searchable runtime event journal, separated from fleet readiness so each
//! operations surface has one clear job.

use std::str::FromStr;

use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Response},
};

use crate::{
    core::workspace_queries,
    events::{EventSeverity, RuntimeEvent, RuntimeEventFilter},
};

use super::super::{html, time, WebState};

const DEFAULT_LIMIT: usize = 100;
const MAX_LIMIT: usize = 500;

#[derive(Default, serde::Deserialize)]
pub struct EventsQuery {
    #[serde(default)]
    severity: String,
    #[serde(default)]
    query: String,
    #[serde(default)]
    limit: String,
}

pub async fn events(State(state): State<WebState>, Query(params): Query<EventsQuery>) -> Response {
    let body = match render(&state, &params) {
        Ok(body) => body,
        Err(error) => format!(
            "{}{}",
            page_header(),
            html::notice("danger", &format!("failed to load event journal: {error}"))
        ),
    };
    Html(html::layout("Events", "events", "", &body)).into_response()
}

fn render(state: &WebState, params: &EventsQuery) -> anyhow::Result<String> {
    let severity = parse_severity(&params.severity)?;
    let limit = parse_limit(&params.limit);
    let filter = RuntimeEventFilter::new(severity, params.query.trim(), limit);
    let total = workspace_queries::count_workspace_events(&state.repository, &filter)?;
    let events = workspace_queries::list_workspace_events(&state.repository, filter)?;

    Ok(format!(
        r#"{head}
{summary}
{filters}
{journal}"#,
        head = page_header(),
        summary = html::cards(&[
            ("Matching", total.to_string()),
            ("Shown", events.len().to_string()),
            (
                "Severity",
                severity.map_or_else(|| "All".to_string(), |value| value.label().to_string()),
            ),
            ("Window", limit.to_string()),
        ]),
        filters = filter_form(params, limit),
        journal = journal(&events),
    ))
}

fn page_header() -> String {
    html::page_head(
        "Events",
        "Audit fleet lifecycle, health, policy, signer-adjacent and workspace activity.",
        r#"<a class="btn" href="/operations">Readiness</a>"#,
    )
}

fn filter_form(params: &EventsQuery, limit: usize) -> String {
    let severities = [
        ("", "All severities"),
        ("info", "Info"),
        ("warning", "Warning"),
        ("critical", "Critical"),
    ]
    .iter()
    .map(|(value, label)| {
        let selected = if *value == params.severity.trim() {
            " selected"
        } else {
            ""
        };
        format!(r#"<option value="{value}"{selected}>{label}</option>"#)
    })
    .collect::<String>();
    let limits = [25_usize, 50, 100, 250, 500]
        .iter()
        .map(|value| {
            let selected = if *value == limit { " selected" } else { "" };
            format!(r#"<option value="{value}"{selected}>{value}</option>"#)
        })
        .collect::<String>();
    format!(
        r#"<form class="filters" method="get" action="/events">
<label class="field"><span>Severity</span><select name="severity">{severities}</select></label>
<label class="field"><span>Search</span><input name="query" value="{query}" placeholder="node, event kind, or message"></label>
<label class="field"><span>Rows</span><select name="limit">{limits}</select></label>
<button type="submit">Apply</button>
</form>"#,
        query = html::escape(&params.query),
    )
}

fn journal(events: &[RuntimeEvent]) -> String {
    if events.is_empty() {
        return html::empty_state(
            "No matching events",
            "The journal has no entries for this filter. Broaden the severity or search text.",
            r#"<a class="btn" href="/events">Clear filters</a>"#,
        );
    }
    let rows = events
        .iter()
        .map(|event| {
            let node = event.node_name.as_deref().unwrap_or("Workspace");
            html::row(&[
                html::raw_cell(&time::time_cell(Some(event.occurred_at_unix))),
                html::raw_cell(&severity_badge(event.severity)),
                html::cell(event.kind.label()),
                html::cell(node),
                html::cell(&event.message),
            ])
        })
        .collect::<Vec<_>>();
    html::table(&["Time", "Severity", "Kind", "Scope", "Message"], &rows)
}

fn severity_badge(severity: EventSeverity) -> String {
    format!(
        r#"<span class="badge event-{}">{}</span>"#,
        severity.label(),
        severity.label(),
    )
}

fn parse_severity(raw: &str) -> anyhow::Result<Option<EventSeverity>> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(None);
    }
    EventSeverity::from_str(raw).map(Some)
}

fn parse_limit(raw: &str) -> usize {
    raw.trim()
        .parse::<usize>()
        .unwrap_or(DEFAULT_LIMIT)
        .clamp(1, MAX_LIMIT)
}

#[cfg(test)]
#[path = "../../../tests/unit/web/events/tests.rs"]
mod tests;
