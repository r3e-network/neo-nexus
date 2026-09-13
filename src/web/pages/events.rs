//! Searchable runtime event journal, separated from fleet readiness so each
//! operations surface has one clear job.

use std::str::FromStr;

use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Response},
};

use crate::events::{EventSeverity, RuntimeEvent, RuntimeEventFilter};

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
    let total = state.workspace.count_events(&filter)?;
    let events = state.workspace.list_events(filter)?;

    let breadcrumb = html::breadcrumb(&[("CloudTrail", "/events"), ("Event history", "")]);

    Ok(format!(
        r#"{breadcrumb}
{head}
{summary}
{filters}
{journal}"#,
        breadcrumb = breadcrumb,
        head = page_header(),
        summary = html::cards(&[
            ("Audit Events", total.to_string()),
            ("Window Limit", limit.to_string()),
            (
                "Severity Filter",
                severity.map_or_else(|| "All".to_string(), |value| value.label().to_string()),
            ),
            (
                "Audit IAM Identity",
                "arn:neo:iam::nexus:operator".to_string()
            ),
        ]),
        filters = filter_form(params, limit),
        journal = journal(&events),
    ))
}

fn page_header() -> String {
    html::page_head(
        "CloudTrail Event History",
        "AWS CloudTrail-grade immutable audit journal recording fleet lifecycle actions, security decisions, and operator activity.",
        r#"<a class="btn" href="/operations">⚙️ SSM OpsCenter</a> <a class="btn" href="/alerts">🚨 CloudWatch Alarms</a>"#,
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
            let event_source = aws_event_source(event.kind.label());
            let identity = if event.message.contains("Hermes") || event.message.contains("probe") {
                "arn:neo:agent::hermes-ai"
            } else {
                "arn:neo:iam::nexus:operator"
            };
            html::row(&[
                html::raw_cell(&time::time_cell(Some(event.occurred_at_unix))),
                html::raw_cell(&severity_badge(event.severity)),
                html::raw_cell(&format!(
                    r#"<span class="badge">{}</span>"#,
                    html::escape(event_source)
                )),
                html::cell(event.kind.label()),
                html::cell(node),
                html::raw_cell(&format!(
                    r#"<span class="mono muted" style="font-size: 11px;">{}</span>"#,
                    html::escape(identity)
                )),
                html::cell(&event.message),
            ])
        })
        .collect::<Vec<_>>();
    html::table(
        &[
            "Event Time",
            "Severity",
            "Event Source",
            "Event Name",
            "Resource Scope",
            "User Identity",
            "Details / Request",
        ],
        &rows,
    )
}

fn aws_event_source(kind: &str) -> &'static str {
    if kind.starts_with("node-") {
        "neo.ec2"
    } else if kind.starts_with("signer-") || kind.contains("key") {
        "neo.kms"
    } else if kind.starts_with("snapshot-") {
        "neo.ebs"
    } else if kind.starts_with("plugin-") {
        "neo.ssm"
    } else if kind.starts_with("alert-") {
        "neo.cloudwatch"
    } else {
        "neo.controlplane"
    }
}

fn severity_badge(severity: EventSeverity) -> String {
    let (class, prefix) = match severity {
        EventSeverity::Critical => ("badge error", "▲ "),
        EventSeverity::Warning => ("badge stopped", "▲ "),
        EventSeverity::Info => ("badge running", "● "),
    };
    format!(
        r#"<span class="{class}">{prefix}{}</span>"#,
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
