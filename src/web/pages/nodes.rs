//! Nodes: the fleet list and the per-node studio — config facts, RPC health
//! trend, and the start/stop/restart controls (plain form posts, so every
//! control works without JavaScript).

use axum::{
    extract::{Path, RawQuery, State},
    response::{Html, IntoResponse, Redirect, Response},
};

use super::super::{fleet::Fleet, html, pages::home::fleet_table, WebState};

pub async fn node_list(State(state): State<WebState>) -> Response {
    match Fleet::load(&state.repository) {
        Ok(fleet) => {
            let table = fleet_table(&fleet);
            let body = format!("<h1>Nodes</h1>\n{table}");
            Html(html::layout("Nodes", "nodes", "", &body)).into_response()
        }
        Err(error) => Html(html::layout(
            "Nodes",
            "nodes",
            &format!("failed to load nodes: {error}"),
            "",
        ))
        .into_response(),
    }
}

pub async fn node_detail(
    State(state): State<WebState>,
    Path(id): Path<String>,
    RawQuery(query): RawQuery,
) -> Response {
    match render_detail(&state, &id) {
        Ok(body) => Html(html::layout(
            "Node",
            "nodes",
            &html::flash(query.as_deref()),
            &body,
        ))
        .into_response(),
        Err(_) => Redirect::to("/nodes").into_response(),
    }
}

fn control_bar(node_id: &str, status: &str) -> String {
    let running = matches!(status, "Running" | "Starting");
    format!(
        r#"<div class="actions">
<form method="post" action="/nodes/{id}/start"><button type="submit">Start</button></form>
<form method="post" action="/nodes/{id}/stop"><button type="submit" {disabled}>Stop</button></form>
<form method="post" action="/nodes/{id}/restart"><button type="submit" {disabled}>Restart</button></form>
</div>"#,
        id = html::escape(node_id),
        disabled = if running { "" } else { "disabled" },
    )
}

fn fact_rows(node: &crate::types::NodeConfig) -> String {
    let facts = [
        ("Type", node.node_type.to_string()),
        ("Network", node.network.to_string()),
        ("Binary", node.binary_path.display().to_string()),
        ("Runtime", node.runtime_version.clone()),
        ("Storage", node.storage_engine.to_string()),
        ("RPC port", node.rpc_port.to_string()),
        ("P2P port", node.p2p_port.to_string()),
        (
            "WS port",
            node.ws_port
                .map_or("—".to_string(), |port| port.to_string()),
        ),
        (
            "PID",
            node.pid.map_or("—".to_string(), |pid| pid.to_string()),
        ),
    ];
    facts
        .iter()
        .map(|(label, value)| {
            format!(
                r#"<tr><th>{label}</th><td>{}</td></tr>"#,
                html::escape(value)
            )
        })
        .collect::<String>()
}

fn render_detail(state: &WebState, id: &str) -> anyhow::Result<String> {
    let node = state
        .repository
        .list_nodes()?
        .into_iter()
        .find(|node| node.id == id)
        .ok_or_else(|| anyhow::anyhow!("node {id} was not found"))?;
    let history =
        crate::core::node_health::node_rpc_health_history(&state.repository, &node.id, 10)?;
    let trend = history
        .iter()
        .map(|record| {
            format!(
                r#"<tr><td>{}</td><td>{}</td><td>{}</td><td class="muted">{}</td></tr>"#,
                record.checked_at_unix,
                html::escape(record.status.label()),
                record
                    .block_count
                    .map_or("—".to_string(), |block| block.to_string()),
                html::escape(&record.message),
            )
        })
        .collect::<String>();
    let body = format!(
        r#"<p><a href="/nodes">&larr; All nodes</a></p>
<h1>{name} {status}</h1>
{controls}
<h2>Configuration</h2>
<table class="facts">{facts}</table>
<h2>RPC health history</h2>
{trend_table}"#,
        name = html::escape(&node.name),
        status = html::status_badge(node.status.label()),
        controls = control_bar(&node.id, node.status.label()),
        facts = fact_rows(&node),
        trend_table = if trend.is_empty() {
            r#"<p class="muted">No RPC probes recorded yet.</p>"#.to_string()
        } else {
            format!(
                r#"<table><tr><th>Checked (unix)</th><th>Status</th><th>Block</th><th>Message</th></tr>{trend}</table>"#
            )
        },
    );
    Ok(body)
}
