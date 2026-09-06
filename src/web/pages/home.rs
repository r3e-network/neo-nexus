//! Fleet Overview: current node state, readiness, activity and real host load.

use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
};

use crate::{
    core::{operations::evaluate_fleet, workspace_queries},
    diagnostics::FleetDiagnostics,
    events::{RuntimeEvent, RuntimeEventFilter},
    metrics::{format_bytes, MetricsCollector, MetricsSnapshot},
};

use super::super::{fleet::Fleet, html, time, WebState};

pub fn fleet_table(fleet: &Fleet, snapshot: &MetricsSnapshot) -> String {
    if fleet.rows.is_empty() {
        return html::empty_state(
            "No nodes registered",
            "Add a node to bring its configuration, process and RPC health into this workspace.",
            r#"<a class="btn primary" href="/nodes/new">Add node</a>"#,
        );
    }
    let rows = fleet
        .rows
        .iter()
        .map(|row| {
            let id = html::urlencoding_lite(&row.node.id);
            let uptime = snapshot
                .node_processes
                .iter()
                .find(|process| process.node_id == row.node.id)
                .map(|process| uptime_label(process.run_time_seconds))
                .unwrap_or_else(|| "—".to_string());
            format!(
                r#"<tr data-node-id="{raw_id}">
<td data-label="Node"><a class="node-name" href="/nodes/{id}">{name}</a><span class="node-meta">{client} · {version}</span></td>
<td data-label="Network">{network}</td>
<td data-label="Status">{status}</td>
<td data-label="RPC health"><span data-node-rpc>{rpc_health}</span></td>
<td data-label="Uptime" class="mono">{uptime}</td>
<td data-label="Actions"><div class="row-actions"><a class="btn small" href="/nodes/{id}">Open</a><a class="btn small" href="/logs?node={id}">Logs</a></div></td>
</tr>"#,
                raw_id = html::escape(&row.node.id),
                name = html::escape(&row.node.name),
                client = html::escape(&row.node.node_type.to_string()),
                version = html::escape(&row.node.runtime_version),
                network = html::escape(&row.node.network.to_string()),
                status = html::status_badge(row.node.status.label()),
                rpc_health = html::escape(&row.rpc_health),
                uptime = html::escape(&uptime),
            )
        })
        .collect::<String>();
    format!(
        r#"<table class="dashboard-table">
<thead><tr><th scope="col">Node</th><th scope="col">Network</th><th scope="col">Status</th><th scope="col">RPC health</th><th scope="col">Uptime</th><th scope="col">Actions</th></tr></thead>
<tbody>{rows}</tbody>
</table>"#
    )
}

pub async fn home(State(state): State<WebState>) -> Response {
    match render(&state) {
        Ok(body) => Html(html::layout("Fleet overview", "home", "", &body)).into_response(),
        Err(error) => Html(html::layout(
            "Fleet overview",
            "home",
            &format!("failed to load the fleet overview: {error}"),
            "",
        ))
        .into_response(),
    }
}

fn render(state: &WebState) -> anyhow::Result<String> {
    let fleet = Fleet::load(&state.repository)?;
    let nodes = state.repository.list_nodes()?;
    let plugin_states = nodes
        .iter()
        .map(|node| {
            state
                .repository
                .list_plugin_states(&node.id)
                .map(|states| (node.id.clone(), states))
        })
        .collect::<anyhow::Result<BTreeMap<_, _>>>()?;
    let diagnostics = evaluate_fleet(&nodes, &plugin_states);
    let events = workspace_queries::list_workspace_events(
        &state.repository,
        RuntimeEventFilter::new(None, "", 5),
    )?;
    let mut collector = MetricsCollector::new(Duration::ZERO);
    let snapshot = collector.refresh(&nodes, Instant::now());
    let table = fleet_table(&fleet, &snapshot);

    Ok(format!(
        r#"{head}
{stats}
<div class="section-head"><h2>Managed nodes</h2><a href="/nodes">Open node inventory</a></div>
{table}
<div class="dashboard-grid">
  {readiness}
  {activity}
</div>
{resources}"#,
        head = html::page_head(
            "Fleet overview",
            "Operate the node fleet from one current, evidence-backed view.",
            r#"<a class="btn" href="/operations">View readiness</a><a class="btn primary" href="/nodes/new">Add node</a>"#,
        ),
        stats = summary_stats(&fleet, &diagnostics),
        readiness = readiness_panel(&diagnostics),
        activity = activity_panel(&events),
        resources = resource_strip(&snapshot),
    ))
}

fn summary_stats(fleet: &Fleet, diagnostics: &FleetDiagnostics) -> String {
    let counts = fleet.count_by_status();
    let attention = diagnostics.warning_count + diagnostics.critical_count;
    let attention_tone = if diagnostics.critical_count > 0 {
        " danger"
    } else if diagnostics.warning_count > 0 {
        " warning"
    } else {
        " positive"
    };
    format!(
        r#"<div class="stat-grid" aria-label="Fleet summary">
<div class="stat"><div class="stat-value">{total}</div><div class="stat-label">Total nodes</div><div class="stat-detail">registered in this workspace</div></div>
<div class="stat positive"><div class="stat-value">{running}</div><div class="stat-label">Running</div><div class="stat-detail">{starting} starting</div></div>
<div class="stat info"><div class="stat-value">{ready}</div><div class="stat-label">Ready</div><div class="stat-detail">no readiness findings</div></div>
<div class="stat{attention_tone}"><div class="stat-value">{attention}</div><div class="stat-label">Attention</div><div class="stat-detail">{critical} critical · {warnings} warning</div></div>
</div>"#,
        total = counts.total,
        running = counts.running,
        starting = counts.starting,
        ready = diagnostics.ready_nodes,
        critical = diagnostics.critical_count,
        warnings = diagnostics.warning_count,
    )
}

fn readiness_panel(diagnostics: &FleetDiagnostics) -> String {
    let issues = diagnostics
        .nodes
        .iter()
        .filter(|node| node.critical_count() > 0 || node.warning_count() > 0)
        .take(4)
        .map(|node| {
            format!(
                r#"<li><a href="/nodes/{id}">{name}</a><span>{critical} critical · {warnings} warning</span></li>"#,
                id = html::urlencoding_lite(&node.node_id),
                name = html::escape(&node.node_name),
                critical = node.critical_count(),
                warnings = node.warning_count(),
            )
        })
        .collect::<String>();
    let issues = if issues.is_empty() {
        r#"<li><span>Every registered node is launch-ready.</span><span class="tone-info">clear</span></li>"#.to_string()
    } else {
        issues
    };
    format!(
        r#"<section class="surface">
<div class="section-head"><h2>Readiness</h2><a href="/operations">Full report</a></div>
<div class="readiness-score"><strong>{score}/100</strong><span>{ready} nodes ready</span></div>
<progress max="100" value="{score}">{score}%</progress>
<ul class="issue-list">{issues}</ul>
</section>"#,
        score = diagnostics.score,
        ready = diagnostics.ready_nodes,
    )
}

fn activity_panel(events: &[RuntimeEvent]) -> String {
    let rows = events
        .iter()
        .map(|event| {
            let node = event.node_name.as_deref().unwrap_or("Workspace");
            format!(
                r#"<li><div><strong>{kind}</strong><p>{node} · {message}</p></div>{time}</li>"#,
                kind = html::escape(event.kind.label()),
                node = html::escape(node),
                message = html::escape(&event.message),
                time = time::time_cell(Some(event.occurred_at_unix)),
            )
        })
        .collect::<String>();
    let rows = if rows.is_empty() {
        r#"<li><div><strong>No recent events</strong><p>The journal is quiet.</p></div></li>"#
            .to_string()
    } else {
        rows
    };
    format!(
        r#"<section class="surface">
<div class="section-head"><h2>Recent activity</h2><a href="/events">All events</a></div>
<ul class="activity-list">{rows}</ul>
</section>"#
    )
}

fn resource_strip(snapshot: &MetricsSnapshot) -> String {
    format!(
        r#"<div class="resource-strip" aria-label="Host resources">
<div class="resource"><strong>{cpu:.1}%</strong><span>Host CPU</span></div>
<div class="resource"><strong>{memory:.1}%</strong><span>Host memory</span></div>
<div class="resource"><strong>{used} / {total}</strong><span>Memory used</span></div>
<div class="resource"><strong>{processes}</strong><span>Host processes</span></div>
</div>"#,
        cpu = snapshot.system.cpu_usage_percent,
        memory = snapshot.system.memory_usage_percent,
        used = format_bytes(snapshot.system.used_memory_bytes),
        total = format_bytes(snapshot.system.total_memory_bytes),
        processes = snapshot.system.process_count,
    )
}

fn uptime_label(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    if days > 0 {
        format!("{days}d {hours:02}h")
    } else if hours > 0 {
        format!("{hours}h {minutes:02}m")
    } else {
        format!("{minutes}m")
    }
}
