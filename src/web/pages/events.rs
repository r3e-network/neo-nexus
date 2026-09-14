//! The journal: what happened, when, and to which node.
//!
//! Two things here were invented. A `User Identity` column derived an actor by
//! **grepping the message text** — `if message.contains("Hermes") ||
//! message.contains("probe")` → `arn:neo:agent::hermes-ai`, else
//! `arn:neo:iam::nexus:operator` — over a `RuntimeEvent` that has no actor
//! field at all, so a human action whose message happened to contain the word
//! "probe" was attributed to the AI agent, and every agent action whose message
//! did not was attributed to the operator. And the page called itself a
//! "CloudTrail-grade immutable audit journal" over a plain table with no hash
//! chain, no signature and no trigger — one that backup import can insert into
//! arbitrarily.
//!
//! What the journal genuinely is: an append-only-by-convention record this
//! workspace writes. It is useful and it is worth reading; it is not evidence.
//!
//! The filters have grown to match. There are 93 event kinds and the page
//! offered no way to pick one, and no node scope at all — so the node page's
//! "view the full journal" link dumped every event in the workspace.

use std::str::FromStr;

use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Response},
};

use crate::events::{EventKind, EventSeverity, RuntimeEvent, RuntimeEventFilter};

use super::super::{html, time, WebState};

const DEFAULT_LIMIT: usize = 100;
const MAX_LIMIT: usize = 500;

#[derive(Default, serde::Deserialize, Clone)]
pub struct EventsQuery {
    #[serde(default)]
    severity: String,
    #[serde(default)]
    query: String,
    #[serde(default)]
    limit: String,
    #[serde(default)]
    kind: String,
    /// The node this view is scoped to. Named `node` to match every other
    /// per-node link in the console (`/logs?node=`, `/plugins?node=`).
    #[serde(default)]
    node: String,
    /// Accepted because the node activity card linked `?q=` against a field
    /// named `query` — so "View the full journal" silently widened to the whole
    /// workspace. The link is fixed; this keeps an old bookmark working.
    #[serde(default)]
    q: String,
}

impl EventsQuery {
    fn search(&self) -> &str {
        if self.query.trim().is_empty() {
            self.q.trim()
        } else {
            self.query.trim()
        }
    }

    fn to_filter(&self) -> anyhow::Result<RuntimeEventFilter> {
        let mut filter = RuntimeEventFilter::new(
            parse_severity(&self.severity)?,
            self.search(),
            parse_limit(&self.limit),
        );
        filter.kind = parse_kind(&self.kind)?;
        if !self.node.trim().is_empty() {
            filter.node_id = Some(self.node.trim().to_string());
        }
        Ok(filter)
    }
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
    let filter = params.to_filter()?;
    let limit = filter.limit;
    let total = state.workspace.count_events(&filter)?;
    let scope = scope_label(state, &filter);
    let events = state.workspace.list_events(filter)?;

    let breadcrumb = html::breadcrumb(&[("NeoNexus", "/"), ("Journal", "")]);

    Ok(format!(
        r#"{breadcrumb}
{head}
{summary}
{filters}
{journal}"#,
        breadcrumb = breadcrumb,
        head = page_header(),
        summary = html::cards(&[
            ("Matching entries", total.to_string()),
            ("Shown", events.len().to_string()),
            ("Scope", scope),
        ]),
        filters = filter_form(state, params, limit),
        journal = journal(&events),
    ))
}

fn page_header() -> String {
    html::page_head(
        "Journal",
        "What this workspace recorded itself doing. Append-only by convention, not by \
         construction: there is no hash chain and no signature, so treat it as a log, not as \
         evidence.",
        r#"<a class="btn" href="/operations">Operations</a> <a class="btn" href="/alerts">Alert routing</a>"#,
    )
}

/// What the current filter narrows to, said in one phrase.
fn scope_label(state: &WebState, filter: &RuntimeEventFilter) -> String {
    let mut parts = Vec::new();
    if let Some(kind) = filter.kind {
        parts.push(kind.label().to_string());
    }
    if let Some(node_id) = &filter.node_id {
        let name = state
            .workspace
            .list_nodes()
            .ok()
            .and_then(|nodes| {
                nodes
                    .into_iter()
                    .find(|node| &node.id == node_id)
                    .map(|node| node.name)
            })
            .unwrap_or_else(|| node_id.clone());
        parts.push(name);
    }
    if let Some(severity) = filter.severity {
        parts.push(severity.label().to_string());
    }
    if parts.is_empty() {
        "everything".to_string()
    } else {
        parts.join(" · ")
    }
}

fn parse_kind(raw: &str) -> anyhow::Result<Option<EventKind>> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(None);
    }
    EventKind::from_str(raw).map(Some)
}

fn filter_form(state: &WebState, params: &EventsQuery, limit: usize) -> String {
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
    // Only the kinds that have actually occurred in this workspace. Offering
    // all 93 would list 51 that no code path constructs, and an operator who
    // picks one and gets nothing cannot tell "it never happened" from "that
    // filter is broken".
    let mut present: Vec<EventKind> = state
        .workspace
        .list_events(RuntimeEventFilter::new(None, "", 500))
        .unwrap_or_default()
        .into_iter()
        .map(|event| event.kind)
        .collect();
    present.sort_by_key(|kind| kind.label());
    present.dedup();
    let kinds = std::iter::once(String::from(r#"<option value="">All kinds</option>"#))
        .chain(present.into_iter().map(|kind| {
            let selected = if kind.label() == params.kind.trim() {
                " selected"
            } else {
                ""
            };
            format!(
                r#"<option value="{value}"{selected}>{value}</option>"#,
                value = html::escape(kind.label())
            )
        }))
        .collect::<String>();

    let nodes = std::iter::once(String::from(r#"<option value="">All nodes</option>"#))
        .chain(
            state
                .workspace
                .list_nodes()
                .unwrap_or_default()
                .into_iter()
                .map(|node| {
                    let selected = if node.id == params.node.trim() {
                        " selected"
                    } else {
                        ""
                    };
                    format!(
                        r#"<option value="{id}"{selected}>{name}</option>"#,
                        id = html::escape(&node.id),
                        name = html::escape(&node.name),
                    )
                }),
        )
        .collect::<String>();

    let export = format!(
        r#"<form method="post" action="/events/export" style="display: inline;">
<input type="hidden" name="severity" value="{severity}">
<input type="hidden" name="query" value="{query}">
<input type="hidden" name="limit" value="{limit}">
<input type="hidden" name="kind" value="{kind}">
<input type="hidden" name="node" value="{node}">
<button type="submit" class="btn small" title="Download exactly the entries this filter selects">Export this view</button>
</form>"#,
        severity = html::escape(params.severity.trim()),
        query = html::escape(params.search()),
        kind = html::escape(params.kind.trim()),
        node = html::escape(params.node.trim()),
    );

    format!(
        r#"<form class="filters" method="get" action="/events">
<label class="field"><span>Severity</span><select name="severity">{severities}</select></label>
<label class="field"><span>Kind</span><select name="kind">{kinds}</select></label>
<label class="field"><span>Node</span><select name="node">{nodes}</select></label>
<label class="field"><span>Search</span><input name="query" value="{query}" placeholder="node, event kind, or message"></label>
<label class="field"><span>Rows</span><select name="limit">{limits}</select></label>
<button type="submit">Apply</button>
</form>
<div style="margin: -6px 0 14px;">{export}</div>"#,
        query = html::escape(params.search()),
    )
}

fn journal(events: &[RuntimeEvent]) -> String {
    if events.is_empty() {
        return html::empty_state(
            "No matching entries",
            "Nothing in the journal matches this filter. Widen the kind, the node or the search text.",
            r#"<a class="btn" href="/events">Clear filters</a>"#,
        );
    }
    let rows = events
        .iter()
        .map(|event| {
            // `node_name` is `None` for workspace-wide events — a settings
            // change, a backup — which is a real distinction and not a gap.
            let scope = event.node_name.as_deref().unwrap_or("workspace");
            html::row(&[
                html::raw_cell(&time::time_cell(Some(event.occurred_at_unix))),
                html::raw_cell(&severity_badge(event.severity)),
                html::cell(event.kind.label()),
                html::cell(scope),
                html::cell(&event.message),
            ])
        })
        .collect::<Vec<_>>();
    html::table(&["When", "Severity", "What", "Scope", "Detail"], &rows)
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

/// Download exactly the entries the current filter selects.
///
/// `EventJournalReporter` has always been able to produce this — it accepts the
/// very `RuntimeEventFilter` the page just applied — and was reachable only
/// from `--export-event-journal`. The operator filing a ticket is the one least
/// likely to have a shell on the host.
///
/// Messages are redacted on the way out by the reporter, the same way `/logs`
/// redacts what it shows.
pub async fn export_events(
    State(state): State<WebState>,
    axum::Form(params): axum::Form<EventsQuery>,
) -> Response {
    use axum::http::{header, HeaderValue, StatusCode};

    let report = (|| -> anyhow::Result<String> {
        let filter = crate::event_journal_report::export_scope(&params.to_filter()?);
        let matched = state.workspace.count_events(&filter)?;
        let events = state.workspace.list_events(filter.clone())?;
        crate::event_journal_report::EventJournalReport::from_events(
            state.workspace_child_dir(""),
            events,
            matched,
            &filter,
            env!("CARGO_PKG_VERSION"),
            time::now_unix(),
        )
        .to_json_text()
    })();

    // `--export-event-journal` never journalled either, so the one operation
    // that takes a copy of the audit trail left no mark in it.
    let _ = state.commands.record_event(crate::events::NewRuntimeEvent {
        node_id: None,
        node_name: None,
        kind: EventKind::EventJournalExported,
        severity: EventSeverity::Info,
        message: match &report {
            Ok(json) => format!("journal exported ({} bytes)", json.len()),
            Err(error) => format!("journal export failed: {error:#}"),
        },
    });

    match report {
        Ok(json) => {
            let mut response = json.into_response();
            let headers = response.headers_mut();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            );
            headers.insert(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_static("attachment; filename=\"neonexus-journal.json\""),
            );
            response
        }
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to export the journal: {error:#}"),
        )
            .into_response(),
    }
}
