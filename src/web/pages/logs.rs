//! Logs: the supervised output of one node, with the same pattern diagnosis the
//! CLI support bundle runs. Read-only viewing is the default — truncating log
//! files requires a deliberate POST with explicit confirmation.

use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Response},
};

use crate::{
    core::{
        operations::format_bytes,
        runtime::{log_path_for, LogDiagnosis, LogLine, LogReader},
    },
    types::NodeConfig,
};

use super::super::{html, WebState};

/// How much of the tail to read. A supervised node can outwrite the page, so the
/// window is stated rather than hidden.
const READ_BUDGET_BYTES: usize = 256 * 1024;
const DEFAULT_VISIBLE_LINES: usize = 200;
const MAX_VISIBLE_LINES: usize = 2_000;

#[derive(Default, serde::Deserialize)]
pub struct LogQuery {
    #[serde(default)]
    node: String,
    #[serde(default)]
    query: String,
    #[serde(default)]
    lines: String,
}

pub async fn logs(State(state): State<WebState>, Query(params): Query<LogQuery>) -> Response {
    let body = match state.workspace.list_nodes() {
        Ok(nodes) => render_body(&state, &nodes, &params),
        Err(error) => html::note(&format!("failed to load nodes: {error}")),
    };
    Html(html::layout("Logs", "logs", "", &body)).into_response()
}

/// The confirmation page for the one destructive action on this page.
///
/// Reaching it is a GET, so the operator arrives here by choosing to, and the
/// deletion itself is the POST that follows. Nothing about the confirmation
/// depends on scripting.
pub async fn clear_logs_confirmation(
    State(state): State<WebState>,
    Query(params): Query<LogQuery>,
) -> Response {
    let nodes = match state.workspace.list_nodes() {
        Ok(nodes) => nodes,
        Err(error) => {
            let body = html::notice("danger", &format!("failed to load nodes: {error}"));
            return Html(html::layout("Clear logs", "logs", "", &body)).into_response();
        }
    };
    // The node the operator was reading is remembered, so the page they land
    // on is about the stream they were looking at. The deletion itself clears
    // the whole directory either way, and the page says so.
    let selected = pick_node(&nodes, &params.node);
    let body = clear_logs_confirmation_page(selected);
    Html(html::layout("Clear logs", "logs", "", &body)).into_response()
}

fn render_body(state: &WebState, nodes: &[NodeConfig], params: &LogQuery) -> String {
    let breadcrumb = html::breadcrumb(&[("NeoNexus", "/"), ("Logs", "")]);
    let head = html::page_head(
        "Logs",
        "Live CloudWatch stdout/stderr stream with pattern diagnosis, error clustering, and high-frequency search filtering.",
        r#"<a class="btn" href="/monitor">CloudWatch Metrics</a> <a class="btn" href="/alerts">🚨 Alarms</a>"#,
    );

    let Some(selected) = pick_node(nodes, &params.node) else {
        return format!(
            "{breadcrumb}\n{head}\n{}",
            html::empty_state(
                "No instances registered",
                "Register a node instance to give this workspace a log stream to read.",
                r#"<a class="btn primary" href="/nodes/new">Launch an instance</a>"#,
            )
        );
    };

    let log_stream_name = format!("aws/ec2/nexus/{}", selected.name);
    let log_stream_breadcrumb =
        html::breadcrumb(&[("NeoNexus", "/"), ("Logs", "/logs"), (&log_stream_name, "")]);

    let visible = visible_lines(&params.lines);
    let log_path = log_path_for(state.workspace_child_dir("logs"), selected);
    format!(
        r#"{breadcrumb}
{head}
<div class="panel" style="margin-bottom: 14px; padding: 12px 16px; background: var(--panel-2); border: 1px solid var(--line); border-radius: 8px;">
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px; flex-wrap: wrap; gap: 8px;">
        <span style="font-size: 11px; font-weight: 600; text-transform: uppercase; color: var(--muted);">Instance Log Streams:</span>
        <div class="mono muted" style="font-size: 11px;">Read from the workspace log directory. Secrets are masked before display.</div>
    </div>
    <div class="actions" style="display: flex; gap: 8px; flex-wrap: wrap; align-items: center;">
        {picker}
        {clear_button}
    </div>
</div>
{filters}
{content}"#,
        breadcrumb = log_stream_breadcrumb,
        head = head,
        picker = node_picker(nodes, selected),
        clear_button = clear_logs_link(),
        filters = html::typed_filter_form(
            "/logs",
            &[("node", &selected.id)],
            &[
                html::FilterControl::Search {
                    label: "Filter pattern",
                    name: "query",
                    value: &params.query,
                    placeholder: "e.g. error, panic, timeout",
                },
                html::FilterControl::Number {
                    label: "Max lines",
                    name: "lines",
                    value: &visible.to_string(),
                    min: 1,
                    max: MAX_VISIBLE_LINES,
                },
            ],
        ),
        content = render_log(selected, &log_path, params, visible),
    )
}

/// Plain links: switching nodes is a GET, so it works with JavaScript off and
/// the result stays bookmarkable. The id travels, the name is what is read.
fn node_picker(nodes: &[NodeConfig], selected: &NodeConfig) -> String {
    nodes
        .iter()
        .map(|node| {
            let current = if node.id == selected.id {
                " primary"
            } else {
                ""
            };
            format!(
                r#"<a class="btn small{current}" href="/logs?node={}">{}</a>"#,
                html::urlencoding_lite(&node.id),
                html::escape(&node.name)
            )
        })
        .collect()
}

fn pick_node<'a>(nodes: &'a [NodeConfig], wanted: &str) -> Option<&'a NodeConfig> {
    let wanted = wanted.trim();
    nodes
        .iter()
        .find(|node| !wanted.is_empty() && (node.name == wanted || node.id == wanted))
        .or_else(|| nodes.first())
}

fn visible_lines(raw: &str) -> usize {
    raw.trim()
        .parse::<usize>()
        .unwrap_or(DEFAULT_VISIBLE_LINES)
        .clamp(1, MAX_VISIBLE_LINES)
}

fn render_log(
    node: &NodeConfig,
    log_path: &std::path::Path,
    params: &LogQuery,
    visible: usize,
) -> String {
    let snapshot = match LogReader::snapshot(log_path, READ_BUDGET_BYTES) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            return html::note(&format!("failed to read {}: {error}", log_path.display()))
        }
    };
    let diagnosis = LogReader::diagnose(&snapshot);
    let matched = LogReader::filtered_lines(&snapshot, &params.query);
    let shown: Vec<&LogLine> = matched.iter().rev().take(visible).rev().collect();

    let stream_header = format!(
        r#"<div class="aws-log-header" style="margin-top: 18px; margin-bottom: 10px;">
            <div style="display: flex; align-items: center; gap: 8px;">
                <span class="aws-log-live-dot"></span>
                <strong style="color: #fff; font-size: 13px;">Log Stream: {}</strong>
                <span class="badge running" style="font-size: 10px;">● Live Tail</span>
            </div>
            <span class="mono muted" style="font-size: 11px;">Path: {}</span>
        </div>"#,
        html::escape(&node.name),
        html::escape(&snapshot.path.display().to_string())
    );

    format!(
        r#"{stream_header}
{tiles}
{diagnosis}
{body}"#,
        stream_header = stream_header,
        tiles = html::cards(&[
            ("Size", format_bytes(snapshot.bytes)),
            ("Lines", snapshot.lines.len().to_string()),
            ("Matching", matched.len().to_string()),
            ("Diagnosis", diagnosis.status.label().to_string()),
        ]),
        diagnosis = diagnosis_panel(&diagnosis),
        body = log_body(node, &shown, snapshot.truncated, params),
    )
}

fn log_body(node: &NodeConfig, lines: &[&LogLine], truncated: bool, params: &LogQuery) -> String {
    if lines.is_empty() {
        // Two different empties: the instance has written nothing, or the
        // search matched nothing. The second has a way out the first does not.
        let (title, body, actions) = if params.query.trim().is_empty() {
            (
                "No log output yet",
                format!("{} has not written anything to its log yet.", node.name),
                String::new(),
            )
        } else {
            (
                "No matching lines",
                format!("No line in {}'s log matches {:?}.", node.name, params.query),
                format!(
                    r#"<a class="btn" href="/logs?node={}">Clear the search</a>"#,
                    html::escape(&node.name)
                ),
            )
        };
        return format!(
            "{}\n{}",
            html::empty_state(title, &body, &actions),
            truncation_note(truncated)
        );
    }
    // A node's log is its own output plus the launch header NeoNexus writes,
    // and either can carry a credential the runtime echoed. The support bundle
    // redacts this same content before it leaves the host; a page that serves
    // it over HTTP has no reason to be laxer.
    let text = lines
        .iter()
        .map(|line| {
            format!(
                "{:>6} | {}\n",
                line.number,
                crate::redaction::redact_sensitive_text(&line.text)
            )
        })
        .collect::<String>();
    format!(
        "{}\n{}",
        html::text_block(&text),
        truncation_note(truncated)
    )
}

fn truncation_note(truncated: bool) -> String {
    if truncated {
        html::note("Older content sits outside the read window; raise the line count or read the file directly.")
    } else {
        String::new()
    }
}

fn diagnosis_panel(diagnosis: &LogDiagnosis) -> String {
    if diagnosis.findings.is_empty() {
        return format!("<h2>Diagnosis</h2>\n{}", html::note(&diagnosis.summary));
    }
    let rows = diagnosis
        .findings
        .iter()
        .map(|finding| {
            html::row(&[
                html::cell(finding.status.label()),
                html::cell(&finding.label),
                html::cell(&finding.line_number.to_string()),
                html::cell(&finding.excerpt),
                html::cell(&finding.recommendation),
            ])
        })
        .collect::<Vec<_>>();
    format!(
        "<h2>Diagnosis</h2>\n{}\n{}",
        html::note(&diagnosis.summary),
        html::table(
            &["Severity", "Pattern", "Line", "Excerpt", "Recommendation"],
            &rows
        )
    )
}

/// A link to the confirmation page. A GET, so the destructive step is never
/// one click away and the intermediate state is bookmarkable and shareable.
///
/// The target is a plain path. The node the operator was reading does not
/// travel in the link: the confirmation page states which stream it refers to
/// and carries the selection in its own form, which is where it belongs — a
/// link whose destination is assembled from a placeholder is a link nothing
/// that reads the source can recognise as pointing here.
fn clear_logs_link() -> String {
    r#"<a class="btn danger" href="/logs/clear">Clear Logs</a>"#.to_string()
}

/// The destructive action needs a confirmation step of its own.
///
/// This used to be a single button whose `onclick="return confirm(...)"` was
/// the only thing standing between the operator and the deletion. The content
/// security policy pins `script-src` to a hash of the one bundled script and
/// allows no `'unsafe-inline'`, so the browser refused to run that inline
/// handler outright — the prompt never appeared and the form posted straight
/// through, deleting every log file in the workspace.
///
/// The confirmation is therefore a server-rendered page, reached by GET and
/// completed by POST. It works with scripting disabled, and it can state what
/// exactly is about to be destroyed.
fn clear_logs_confirmation_page(selected: Option<&NodeConfig>) -> String {
    let breadcrumb = html::breadcrumb(&[("NeoNexus", "/"), ("Logs", "/logs"), ("Clear", "")]);
    let head = html::page_head(
        "Clear all logs?",
        "This permanently deletes every log file in the workspace logs directory.",
        "",
    );
    // The action clears the whole directory, so the page must not read as
    // though it were about the stream of a single instance.
    let scope = match selected {
        Some(node) => format!(
            "Every log file under the workspace logs directory is truncated, for all instances — not \
             just {name}. The running nodes keep their file handles, so the next write may recreate \
             an empty file. This cannot be undone, and there is no archive of the deleted output.",
            name = html::escape(&node.name)
        ),
        None => "Every log file under the workspace logs directory is truncated, for every instance. \
                  No instance is registered, so nothing is writing to it right now, but any file \
                  left there is deleted. This cannot be undone."
            .to_string(),
    };
    let cancel = match selected {
        Some(node) => format!("/logs?node={}", html::urlencoding_lite(&node.id)),
        None => "/logs".to_string(),
    };
    // The node travels with the POST, so the operator lands back on the
    // stream they were reading rather than on the first one.
    let carried = match selected {
        Some(node) => format!(
            r#"<input type="hidden" name="node" value="{}">"#,
            html::escape(&node.id)
        ),
        None => String::new(),
    };
    format!(
        r#"{breadcrumb}
{head}
{detail}
<form method="post" action="/logs">
<div class="form-actions">
<button class="danger" type="submit">Clear all log files</button>
<a class="btn" href="{cancel}">Cancel</a>
{pending}
</div>
{carried}
</form>"#,
        breadcrumb = breadcrumb,
        head = head,
        detail = html::notice("danger", &scope),
        cancel = cancel,
        carried = carried,
        // Truncating every log file in the workspace is not instant, and the
        // operator must not click twice wondering whether the first landed.
        pending = html::pending_marker("Clearing…"),
    )
}
