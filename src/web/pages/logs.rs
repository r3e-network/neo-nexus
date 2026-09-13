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

fn render_body(state: &WebState, nodes: &[NodeConfig], params: &LogQuery) -> String {
    let breadcrumb = html::breadcrumb(&[
        ("CloudWatch", "/monitor"),
        ("Logs", "/logs"),
        ("Log groups", "/logs"),
    ]);
    let head = html::page_head(
        "Logs",
        "Live CloudWatch stdout/stderr stream with pattern diagnosis, error clustering, and high-frequency search filtering.",
        r#"<a class="btn" href="/monitor">CloudWatch Metrics</a> <a class="btn" href="/alerts">🚨 Alarms</a>"#,
    );

    let Some(selected) = pick_node(nodes, &params.node) else {
        return format!(
            "{breadcrumb}\n{head}\n{}",
            html::note("No instances are registered yet, so there are no log streams to read.")
        );
    };

    let log_stream_name = format!("aws/ec2/nexus/{}", selected.name);
    let log_stream_breadcrumb = html::breadcrumb(&[
        ("CloudWatch", "/monitor"),
        ("Logs", "/logs"),
        ("Log groups", "/logs"),
        (&log_stream_name, ""),
    ]);

    let visible = visible_lines(&params.lines);
    let log_path = log_path_for(state.workspace_child_dir("logs"), selected);
    format!(
        r#"{breadcrumb}
{head}
<div class="panel" style="margin-bottom: 14px; padding: 12px 16px; background: var(--panel-2); border: 1px solid var(--line); border-radius: 8px;">
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px; flex-wrap: wrap; gap: 8px;">
        <span style="font-size: 11px; font-weight: 600; text-transform: uppercase; color: var(--muted);">Instance Log Streams:</span>
        <div class="mono muted" style="font-size: 11px;">Log Group: /aws/ec2/nexus · Region: nexus-global (mesh-1a)</div>
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
        clear_button = clear_logs_button(&selected.id),
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
        let reason = if params.query.trim().is_empty() {
            format!("{} has written nothing yet.", node.name)
        } else {
            format!("No lines match {:?}.", params.query)
        };
        return format!("{}\n{}", html::note(&reason), truncation_note(truncated));
    }
    let text = lines
        .iter()
        .map(|line| format!("{:>6} | {}\n", line.number, line.text))
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

/// Provide a destructive action button with inline confirmation form.
/// The form POSTs to /logs and includes hidden inputs for node context,
/// ensuring the redirect preserves the selected node after clearing.
fn clear_logs_button(node_id: &str) -> String {
    format!(
        r#"<form method="POST" action="/logs" style="display:inline; margin-left:8px;">
  <button type="submit" class="btn danger" onclick="return confirm('This will permanently delete all .log files in the workspace logs directory. Are you sure you want to continue?');">Clear Logs</button>
  <input type="hidden" name="node" value="{}">
</form>"#,
        html::urlencoding_lite(node_id)
    )
}
