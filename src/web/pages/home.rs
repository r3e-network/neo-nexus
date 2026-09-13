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
    core::operations::evaluate_fleet,
    diagnostics::FleetDiagnostics,
    events::{RuntimeEvent, RuntimeEventFilter},
    metrics::{format_bytes, MetricsCollector, MetricsSnapshot},
};

use super::super::{fleet::Fleet, html, time, WebState};

pub fn fleet_table(fleet: &Fleet, snapshot: &MetricsSnapshot) -> String {
    if fleet.rows.is_empty() {
        return html::empty_state(
            "No instances registered",
            "Launch an EC2 node instance to bring its configuration, process supervision, and RPC telemetry into this workspace.",
            r#"<a class="btn primary" href="/nodes/new">+ Launch Instance</a>"#,
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
            let status_badge = if row.node.status.is_running() {
                r#"<span class="badge running">● 2/2 passed</span>"#
            } else if row.node.status == crate::types::NodeStatus::Stopped {
                r#"<span class="badge stopped">⚪ 0/2 stopped</span>"#
            } else {
                r#"<span class="badge danger">▲ 1/2 impaired</span>"#
            };
            format!(
                r#"<tr data-node-id="{raw_id}">
<td data-label="Instance"><div><a class="node-name" href="/nodes/{id}" style="font-weight: 600;">{name}</a></div><div class="muted mono" style="font-size: 11px;">{raw_id}</div></td>
<td data-label="Type"><span class="badge">{client}</span> <span class="badge">{network}</span></td>
<td data-label="Status">{status}</td>
<td data-label="Status check">{status_badge}</td>
<td data-label="Availability Zone"><span class="mono" style="font-size: 12px;">nexus-az-1a</span></td>
<td data-label="Telemetry"><span data-node-rpc>{rpc_health}</span></td>
<td data-label="Uptime" class="mono">{uptime}</td>
<td data-label="Actions"><div class="row-actions"><a class="btn small primary" href="/nodes/{id}">Studio</a><a class="btn small" href="/logs?node={id}">Logs</a></div></td>
</tr>"#,
                raw_id = html::escape(&row.node.id),
                name = html::escape(&row.node.name),
                client = html::escape(&row.node.node_type.to_string()),
                network = html::escape(&row.node.network.to_string()),
                status = html::status_badge(row.node.status.label()),
                status_badge = status_badge,
                rpc_health = html::escape(&row.rpc_health),
                uptime = html::escape(&uptime),
            )
        })
        .collect::<String>();
    format!(
        r#"<table class="dashboard-table">
<thead><tr><th scope="col">Instance</th><th scope="col">Engine &amp; Network</th><th scope="col">State</th><th scope="col">Status check</th><th scope="col">Availability Zone</th><th scope="col">Telemetry</th><th scope="col">Uptime</th><th scope="col">Actions</th></tr></thead>
<tbody>{rows}</tbody>
</table>"#
    )
}

pub async fn home(State(state): State<WebState>) -> Response {
    match render(&state) {
        Ok(body) => Html(html::layout("Console Home", "home", "", &body)).into_response(),
        Err(error) => Html(html::layout(
            "Console Home",
            "home",
            &format!("failed to load the fleet overview: {error}"),
            "",
        ))
        .into_response(),
    }
}

fn render(state: &WebState) -> anyhow::Result<String> {
    let fleet = Fleet::load(&state.workspace)?;
    let nodes = state.workspace.list_nodes()?;
    let plugin_states = nodes
        .iter()
        .map(|node| {
            state
                .workspace
                .list_plugin_states(&node.id)
                .map(|states| (node.id.clone(), states))
        })
        .collect::<anyhow::Result<BTreeMap<_, _>>>()?;
    let diagnostics = evaluate_fleet(&nodes, &plugin_states);
    let events = state
        .workspace
        .list_events(RuntimeEventFilter::new(None, "", 5))?;
    let mut collector = MetricsCollector::new(Duration::ZERO);
    let snapshot = collector.refresh(&nodes, Instant::now());
    let table = fleet_table(&fleet, &snapshot);

    let breadcrumb = html::breadcrumb(&[("AWS Console", "/"), ("Console Home", "/")]);

    let quick_services = r#"<div class="panel" style="margin-bottom: 16px; padding: 14px 18px; background: var(--panel-2); border: 1px solid var(--line); border-radius: 8px;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;">
            <strong style="font-size: 13px; text-transform: uppercase; color: var(--muted); letter-spacing: 0.5px;">Recently Visited Services</strong>
            <span class="mono muted" style="font-size: 11px;">Region: nexus-global (mesh-1a) · Account: 0123-4567-8901</span>
        </div>
        <div style="display: flex; gap: 8px; flex-wrap: wrap;">
            <a href="/nodes" class="btn small" style="display: inline-flex; align-items: center; gap: 6px;"><span>💻</span> EC2 Instances</a>
            <a href="/nodes/new" class="btn small primary" style="display: inline-flex; align-items: center; gap: 6px;"><span>➕</span> Launch Instance</a>
            <a href="/monitor" class="btn small" style="display: inline-flex; align-items: center; gap: 6px;"><span>📊</span> CloudWatch Metrics</a>
            <a href="/alerts" class="btn small" style="display: inline-flex; align-items: center; gap: 6px;"><span>🚨</span> CloudWatch Alarms</a>
            <a href="/operations" class="btn small" style="display: inline-flex; align-items: center; gap: 6px;"><span>🛠️</span> SSM OpsCenter</a>
            <a href="/events" class="btn small" style="display: inline-flex; align-items: center; gap: 6px;"><span>📜</span> CloudTrail Audit</a>
            <a href="/snapshots" class="btn small" style="display: inline-flex; align-items: center; gap: 6px;"><span>💾</span> EBS Snapshots</a>
            <a href="/signer" class="btn small" style="display: inline-flex; align-items: center; gap: 6px;"><span>🔒</span> KMS Key Management</a>
            <a href="/settings/api-tokens" class="btn small" style="display: inline-flex; align-items: center; gap: 6px;"><span>🔑</span> IAM Credentials</a>
        </div>
    </div>"#;

    let health_widgets = r#"<div class="grid" style="grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 14px; margin-bottom: 16px;">
            <div class="panel" style="padding: 14px; background: var(--panel-2); border: 1px solid var(--line); border-radius: 8px;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                    <strong style="font-size: 13px;">AWS Health Dashboard</strong>
                    <span class="badge running" style="font-size: 10px;">● Operational</span>
                </div>
                <div class="muted" style="font-size: 12px; margin-bottom: 8px;">All blockchain subsystem services and supervisor daemons are operating normally.</div>
                <div style="display: flex; gap: 16px; font-size: 12px;">
                    <div><span class="muted">Open issues:</span> <strong style="color: var(--jade);">0</strong></div>
                    <div><span class="muted">Scheduled changes:</span> <strong>0</strong></div>
                    <div><span class="muted">Other notifications:</span> <strong>0</strong></div>
                </div>
            </div>
            <div class="panel" style="padding: 14px; background: var(--panel-2); border: 1px solid var(--line); border-radius: 8px;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                    <strong style="font-size: 13px;">CloudWatch Alarms Status</strong>
                    <a href="/alerts" class="muted" style="font-size: 11px;">Manage alarms ›</a>
                </div>
                <div class="muted" style="font-size: 12px; margin-bottom: 8px;">Automated fleet metric threshold monitors and SNS pager routing rules.</div>
                <div style="display: flex; gap: 16px; font-size: 12px;">
                    <div><span class="muted">In ALARM:</span> <strong style="color: var(--jade);">0</strong></div>
                    <div><span class="muted">OK:</span> <strong style="color: var(--jade);">4</strong></div>
                    <div><span class="muted">Insufficient data:</span> <strong>0</strong></div>
                </div>
            </div>
        </div>"#.to_string();

    Ok(format!(
        r#"{breadcrumb}
{head}
{quick_services}
{health_widgets}
{stats}
<div class="section-head"><h2>EC2 Instance Inventory</h2><div style="display: flex; gap: 8px;"><a class="btn small" href="/nodes">View All Instances</a><a class="btn small primary" href="/nodes/new">+ Launch Instance</a></div></div>
{table}
<div class="dashboard-grid">
  {readiness}
  {activity}
</div>
{resources}"#,
        breadcrumb = breadcrumb,
        head = html::page_head(
            "AWS Management Console · Global Command Center",
            "Fleet orchestration, real-time node supervision, automated self-healing, and hyperscaler telemetry.",
            r#"<a class="btn" href="/api/fleet/iac?format=cloudformation" download="fleet-cloudformation.yaml" title="Export AWS CloudFormation stack">☁️ Export CloudFormation</a> <a class="btn primary" href="/nodes/new">+ Launch Instance</a>"#,
        ),
        quick_services = quick_services,
        health_widgets = health_widgets,
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
<div class="stat"><div class="stat-value">{total}</div><div class="stat-label">Total instances</div><div class="stat-detail">registered in this VPC</div></div>
<div class="stat positive"><div class="stat-value">{running}</div><div class="stat-label">Running</div><div class="stat-detail">{starting} pending</div></div>
<div class="stat info"><div class="stat-value">{ready}</div><div class="stat-label">Health passed</div><div class="stat-detail">2/2 checks passing</div></div>
<div class="stat{attention_tone}"><div class="stat-value">{attention}</div><div class="stat-label">OpsFindings</div><div class="stat-detail">{critical} critical · {warnings} warning</div></div>
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
