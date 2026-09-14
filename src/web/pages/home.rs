//! What to look at first.
//!
//! This page used to open with a panel captioned "AWS Health Dashboard"
//! reporting `● Operational` and `Open issues: 0` — literals, computed from
//! nothing — above a tile reading `2/2 checks passing` that was mapped straight
//! from `is_running()`, and an instance table whose "RPC health" column showed
//! the result of a probe that ran against one node per second. An operator
//! could open this console during an incident, read a screen of green, and stop
//! looking.
//!
//! What replaces it is an **attention queue**: the nodes whose state needs
//! someone, worst first, each with the reason it is there and one thing to do
//! about it. When the queue is empty the page says what it checked and when,
//! rather than asserting that all is well — the two are not the same claim, and
//! only the first is one this workspace can make.

use std::collections::BTreeMap;

use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
};

use crate::{
    core::{
        node_health::{NodeChainView, NodeHealth},
        operations::evaluate_fleet,
    },
    diagnostics::FleetDiagnostics,
    events::{RuntimeEvent, RuntimeEventFilter},
    observe::HealthState,
    types::NodeConfig,
};

use super::super::{chain_state_view as chain_view, html, time, WebState};

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
    let now = time::now_unix();
    let nodes = state.workspace.list_nodes()?;
    let views = state.workspace.fleet_chain_view(&nodes, now)?;
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
        .list_events(RuntimeEventFilter::new(None, "", 6))?;

    Ok(format!(
        r#"{breadcrumb}
{head}
{stats}
{queue}
<div class="section-head"><h2>Fleet</h2><div style="display: flex; gap: 8px;"><a class="btn small" href="/nodes">All nodes</a><a class="btn small primary" href="/nodes/new">+ Add node</a></div></div>
{table}
<div class="dashboard-grid">
  {readiness}
  {activity}
</div>"#,
        breadcrumb = html::breadcrumb(&[("NeoNexus", "/"), ("Console Home", "/")]),
        head = html::page_head(
            "Fleet overview",
            "What every node in this workspace is doing on the chain it joined, \
             and what needs someone.",
            r#"<a class="btn" href="/monitor">Monitoring</a> <a class="btn primary" href="/nodes/new">+ Add node</a>"#,
        ),
        stats = summary_stats(&nodes, &views, now),
        queue = attention_queue(&nodes, &views, now),
        table = fleet_table(&nodes, &views, now),
        readiness = readiness_panel(&diagnostics),
        activity = activity_panel(&events, now),
    ))
}

/// Counts of what the workspace knows, with "not judged" kept separate from
/// "judged and fine".
///
/// The tile this replaces read `2/2 checks passing` for every running node,
/// including nodes whose RPC had never once answered. Here a node that has not
/// been judged is counted as not judged, because it is.
fn summary_stats(nodes: &[NodeConfig], views: &[NodeChainView], now: u64) -> String {
    let running = nodes.iter().filter(|node| node.status.is_running()).count();
    let mut healthy = 0;
    let mut attention = 0;
    let mut unjudged = 0;
    let mut oldest_verdict: Option<u64> = None;
    for view in views {
        match view.health.as_ref() {
            None => unjudged += 1,
            Some(health) => {
                if health.state == HealthState::Healthy {
                    healthy += 1;
                }
                if health.state.needs_attention() {
                    attention += 1;
                }
                let age = health.evaluated_seconds_ago(now);
                oldest_verdict = Some(oldest_verdict.map_or(age, |worst: u64| worst.max(age)));
            }
        }
    }
    let tone = if attention > 0 {
        " danger"
    } else {
        " positive"
    };
    let freshness = match oldest_verdict {
        Some(age) => format!("oldest verdict {} old", chain_view::duration_label(age)),
        None => "nothing judged yet".to_string(),
    };
    format!(
        r#"<div class="stat-grid" aria-label="Fleet summary">
<div class="stat"><div class="stat-value">{total}</div><div class="stat-label">Nodes</div><div class="stat-detail">{running} with a process running</div></div>
<div class="stat{tone}"><div class="stat-value">{attention}</div><div class="stat-label">Need attention</div><div class="stat-detail">unreachable, stalled, isolated or degraded</div></div>
<div class="stat positive"><div class="stat-value">{healthy}</div><div class="stat-label">Healthy</div><div class="stat-detail">answering and keeping up</div></div>
<div class="stat"><div class="stat-value">{unjudged}</div><div class="stat-label">Not yet judged</div><div class="stat-detail">{freshness}</div></div>
</div>"#,
        total = nodes.len(),
        freshness = html::escape(&freshness),
    )
}

/// The nodes that need someone, worst first.
///
/// Ordered by the state machine's own precedence, which is the order the guard
/// chain evaluates: `Unreachable` before `Stalled` before `Isolated` before
/// `Degraded`. Within a state, the node that has been there longest leads —
/// it has been broken longest and is the one nobody has looked at.
fn attention_queue(nodes: &[NodeConfig], views: &[NodeChainView], now: u64) -> String {
    let namer = |node_id: &str| {
        nodes
            .iter()
            .find(|node| node.id == node_id)
            .map_or_else(|| node_id.to_string(), |node| node.name.clone())
    };
    // Paired rather than filtered, so the verdict a row renders is the one the
    // filter matched on — there is no second lookup that could come back empty
    // and no unwrap to stand in for the pairing.
    let mut queue: Vec<(&NodeChainView, &NodeHealth)> = views
        .iter()
        .filter_map(|view| view.health.as_ref().map(|health| (view, health)))
        .filter(|(_, health)| health.state.needs_attention())
        .collect();
    queue.sort_by_key(|(_, health)| (health.state, health.since_unix));

    if queue.is_empty() {
        return quiet_queue(views, now);
    }

    let rows = queue
        .iter()
        .map(|(view, health)| {
            let id = html::urlencoding_lite(&view.node_id);
            format!(
                r#"<li>
<div>
  <div style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">{badge}<a href="/nodes/{id}" style="font-weight: 650;">{name}</a><span class="muted" style="font-size: 11px;">for {held}</span></div>
  <p>{reason}{cause}</p>
</div>
{next}
</li>"#,
                badge = chain_view::health_badge(health.state),
                name = html::escape(&namer(&view.node_id)),
                held = html::escape(&chain_view::duration_label(health.held_for_seconds(now))),
                reason = html::escape(&health.reason),
                cause = health
                    .cause
                    .as_deref()
                    .map(|cause| format!(" {}", html::escape(cause)))
                    .unwrap_or_default(),
                next = next_action(health),
            )
        })
        .collect::<String>();

    format!(
        r#"<section class="surface" style="margin-bottom: 16px;">
<div class="section-head"><h2>Needs attention</h2><span class="muted" style="font-size: 11px;">{count} of {total} nodes</span></div>
<ul class="activity-list">{rows}</ul>
</section>"#,
        count = queue.len(),
        total = views.len(),
    )
}

fn next_action(health: &NodeHealth) -> String {
    match &health.next {
        crate::observe::NextStep::Here { label, href } => format!(
            r#"<a class="btn small primary" href="{href}">{label}</a>"#,
            href = html::escape(href),
            label = html::escape(label),
        ),
        crate::observe::NextStep::External { text } => format!(
            r#"<span class="muted" style="font-size: 11px; max-width: 220px;">{}</span>"#,
            html::escape(text)
        ),
    }
}

/// What an empty queue says.
///
/// Not "all systems operational". The workspace can state what it checked and
/// when it last checked; it cannot state that nothing is wrong with something
/// it has not looked at, and the difference is the whole reason this page was
/// rewritten.
fn quiet_queue(views: &[NodeChainView], now: u64) -> String {
    if views.is_empty() {
        return html::empty_state(
            "No nodes registered",
            "Add a node to bring its configuration, process supervision and chain state into this workspace.",
            r#"<a class="btn primary" href="/nodes/new">+ Add node</a>"#,
        );
    }
    let judged = views.iter().filter(|view| view.health.is_some()).count();
    let newest = views
        .iter()
        .filter_map(|view| view.health.as_ref())
        .map(|health| health.evaluated_at_unix)
        .max();
    let checked = match newest {
        Some(at) => format!("last judged {}", time::relative(at, now)),
        None => "nothing has been judged yet".to_string(),
    };
    format!(
        r#"<section class="surface" style="margin-bottom: 16px;">
<div class="section-head"><h2>Needs attention</h2><span class="muted" style="font-size: 11px;">{checked}</span></div>
<p class="muted" style="margin: 0;">Nothing is queued. {judged} of {total} nodes have a current verdict; the rest have not been judged, which is not the same as being well.</p>
</section>"#,
        checked = html::escape(&checked),
        total = views.len(),
    )
}

/// Every node, on its two independent axes.
///
/// **Process** is what the supervisor sees; **chain** is what the node says
/// when asked. `Running` and `Stalled` together is a legal, expensive state and
/// the failure this workspace exists to catch, so the two are never fused into
/// one badge.
pub fn fleet_table(nodes: &[NodeConfig], views: &[NodeChainView], now: u64) -> String {
    if nodes.is_empty() {
        return html::empty_state(
            "No nodes registered",
            "Add a node to bring its configuration, process supervision and chain state into this workspace.",
            r#"<a class="btn primary" href="/nodes/new">+ Add node</a>"#,
        );
    }
    let rows = nodes
        .iter()
        .map(|node| {
            let view = views.iter().find(|view| view.node_id == node.id);
            let id = html::urlencoding_lite(&node.id);
            format!(
                r#"<tr data-node-id="{raw_id}">
<td data-label="Node"><div><a class="node-name" href="/nodes/{id}" style="font-weight: 600;">{name}</a></div><div class="muted mono" style="font-size: 11px;">{client} · {network}</div></td>
<td data-label="Process">{status}</td>
<td data-label="Chain health">{health}</td>
<td data-label="Height">{chain}</td>
<td data-label="Actions"><div class="row-actions"><a class="btn small primary" href="/nodes/{id}">Open</a><a class="btn small" href="/logs?node={id}">Logs</a></div></td>
</tr>"#,
                raw_id = html::escape(&node.id),
                name = html::escape(&node.name),
                client = html::escape(&node.node_type.to_string()),
                network = html::escape(&node.network.to_string()),
                status = html::status_badge(node.status.label()),
                health = view.map_or_else(chain_view::not_judged_badge, |view| {
                    chain_view::health_cell(view, now)
                }),
                chain = view
                    .map(chain_view::chain_cell)
                    .unwrap_or_else(|| r#"<span class="muted">not checked yet</span>"#.to_string()),
            )
        })
        .collect::<String>();
    format!(
        r#"<table class="dashboard-table">
<thead><tr><th scope="col">Node</th><th scope="col">Process</th><th scope="col">Chain health</th><th scope="col">Height</th><th scope="col">Actions</th></tr></thead>
<tbody>{rows}</tbody>
</table>"#
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
<div class="section-head"><h2>Configuration readiness</h2><a href="/operations">Full report</a></div>
<p class="muted" style="margin: 0 0 8px; font-size: 12px;">Whether each node could start cleanly — ports, binaries, plugins. Separate from chain health, which is what a node says once it has started.</p>
<div class="readiness-score"><strong>{score}/100</strong><span>{ready} nodes ready</span></div>
<progress max="100" value="{score}">{score}%</progress>
<ul class="issue-list">{issues}</ul>
</section>"#,
        score = diagnostics.score,
        ready = diagnostics.ready_nodes,
    )
}

fn activity_panel(events: &[RuntimeEvent], now: u64) -> String {
    let rows = events
        .iter()
        .map(|event| {
            let node = event.node_name.as_deref().unwrap_or("Workspace");
            format!(
                r#"<li><div><strong>{kind}</strong><p>{node} · {message}</p></div><span class="muted">{when}</span></li>"#,
                kind = html::escape(event.kind.label()),
                node = html::escape(node),
                message = html::escape(&event.message),
                when = html::escape(&time::relative(event.occurred_at_unix, now)),
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
