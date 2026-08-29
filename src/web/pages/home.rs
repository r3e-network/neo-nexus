//! Home: fleet counts, host pressure, and the fleet table.

use std::time::{Duration, Instant};

use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
};

use crate::metrics::{format_bytes, MetricsCollector, MetricsSnapshot};

use super::super::{fleet::Fleet, html, WebState};

pub fn fleet_table(fleet: &Fleet) -> String {
    if fleet.rows.is_empty() {
        return r#"<div class="flash">No nodes are registered yet. Import a backup with
<code>--import-backup</code> or create nodes in the workspace database.</div>"#
            .to_string();
    }
    let rows = fleet
        .rows
        .iter()
        .map(|row| {
            format!(
                r#"<tr data-node-id="{id}">
<td><a href="/nodes/{id}">{name}</a></td>
<td>{node_type}</td>
<td>{network}</td>
<td>{status}</td>
<td>{rpc_port}</td>
<td class="muted" data-node-rpc>{rpc_health}</td>
</tr>"#,
                id = html::escape(&row.node.id),
                name = html::escape(&row.node.name),
                node_type = html::escape(&row.node.node_type.to_string()),
                network = html::escape(&row.node.network.to_string()),
                status = html::status_badge(row.node.status.label()),
                rpc_port = row.node.rpc_port,
                rpc_health = html::escape(&row.rpc_health),
            )
        })
        .collect::<String>();
    format!(
        r#"<table>
<tr><th>Node</th><th>Type</th><th>Network</th><th>Status</th><th>RPC</th><th>RPC health</th></tr>
{rows}
</table>"#
    )
}

fn summary_cards(fleet: &Fleet, snapshot: &MetricsSnapshot) -> String {
    let counts = fleet.count_by_status();
    let memory = format!(
        "{:.0}% of {}",
        snapshot.system.memory_usage_percent,
        format_bytes(snapshot.system.total_memory_bytes),
    );
    [
        (counts.total.to_string(), "Nodes"),
        (counts.running.to_string(), "Running"),
        (counts.stopped.to_string(), "Stopped"),
        (counts.error.to_string(), "Error"),
        (
            format!("{:.0}%", snapshot.system.cpu_usage_percent),
            "Host CPU",
        ),
        (memory, "Host memory"),
    ]
    .iter()
    .map(|(num, lbl)| {
        format!(
            r#"<div class="card"><div class="num">{num}</div><div class="lbl">{lbl}</div></div>"#
        )
    })
    .collect::<String>()
}

pub async fn home(State(state): State<WebState>) -> Response {
    match render(&state) {
        Ok(body) => Html(html::layout("Home", "home", "", &body)).into_response(),
        Err(error) => Html(html::layout(
            "Home",
            "home",
            &format!("failed to load the fleet: {error}"),
            "",
        ))
        .into_response(),
    }
}

fn render(state: &WebState) -> anyhow::Result<String> {
    let fleet = Fleet::load(&state.repository)?;
    let nodes = state.repository.list_nodes()?;
    let mut collector = MetricsCollector::new(Duration::ZERO);
    let snapshot = collector.refresh(&nodes, Instant::now());
    let counts = fleet.count_by_status();
    Ok(format!(
        r#"<h1>Fleet overview</h1>
<div class="cards">{cards}</div>
<h2>Nodes</h2>
{table}
<p class="muted">{starting} starting · {error_count} in error — the page polls status every 5 seconds.</p>"#,
        cards = summary_cards(&fleet, &snapshot),
        table = fleet_table(&fleet),
        starting = counts.starting,
        error_count = counts.error,
    ))
}
