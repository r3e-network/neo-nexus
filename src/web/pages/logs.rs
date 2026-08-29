//! Logs: the supervised output of one node, with the same pattern diagnosis the
//! CLI support bundle runs. Deliberately read-only — truncating a log stays a
//! conscious operator action rather than a button next to the fleet view.

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
    let body = match state.repository.list_nodes() {
        Ok(nodes) => render_body(&state, &nodes, &params),
        Err(error) => html::note(&format!("failed to load nodes: {error}")),
    };
    Html(html::layout("Logs", "logs", "", &body)).into_response()
}

fn render_body(state: &WebState, nodes: &[NodeConfig], params: &LogQuery) -> String {
    let Some(selected) = pick_node(nodes, &params.node) else {
        return format!(
            "<h1>Logs</h1>\n{}",
            html::note("No nodes are registered yet, so there are no logs to read.")
        );
    };

    let visible = visible_lines(&params.lines);
    let log_path = log_path_for(state.workspace_child_dir("logs"), selected);
    format!(
        r#"<h1>Logs</h1>
<div class="actions">{picker}</div>
{filters}
{content}"#,
        picker = node_picker(nodes, selected),
        filters = html::filter_form(
            "/logs",
            &[
                ("node", &selected.name),
                ("query", &params.query),
                ("lines", &visible.to_string()),
            ],
        ),
        content = render_log(selected, &log_path, params, visible),
    )
}

/// Plain links: switching nodes is a GET, so it works with JavaScript off and
/// the result stays bookmarkable.
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
                r#"<a class="btn{current}" href="/logs?node={}">{}</a>"#,
                html::urlencoding_lite(&node.name),
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

    format!(
        r#"<h2>{path}</h2>
{tiles}
{diagnosis}
{body}"#,
        path = html::escape(&snapshot.path.display().to_string()),
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
