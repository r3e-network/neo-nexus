//! What the chain says about this workspace's nodes, headlessly.
//!
//! The CLI could already ask *one endpoint* whether it was answering
//! (`--rpc-health`), which is a liveness probe against a URL and knows nothing
//! about the node it belongs to. What it could not do was ask the question an
//! operator actually has — **is anything wrong right now, and with what** —
//! without opening a browser.
//!
//! Both commands exit non-zero when something needs attention, so a cron job or
//! a deployment gate can act on the answer rather than parse it.

use super::super::*;
use crate::core::node_health::{
    duration_label, fleet_chain_view, node_health_timeline, HealthState, NextStep, NodeChainView,
    StallScope, TIMELINE_LENGTH,
};

use super::report::truncate_node_name;
use super::workspace::{node_by_name, open_workspace};

/// `--fleet-health <db>`: every node's verdict, worst first.
pub(in crate::cli::actions) fn fleet_health_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 3, "--fleet-health")?;
    let (views, nodes) = load(&args[2])?;
    let namer = |node_id: &str| name_of(&nodes, node_id);

    if views.is_empty() {
        return Ok(CliAction::PrintWithExitCode {
            exit_code: 0,
            text: "No nodes in the workspace.".to_string(),
        });
    }

    let now = current_unix_time()?;
    let mut ordered: Vec<&NodeChainView> = views.iter().collect();
    ordered.sort_by_key(|view| {
        (
            view.health.as_ref().map(|health| health.state),
            view.health.as_ref().map(|health| health.since_unix),
        )
    });

    let mut lines = vec![format!(
        "{:<24} {:<12} {:<10} {:>12}  {}",
        "NAME", "HEALTH", "HELD", "HEIGHT", "WHY"
    )];
    for view in &ordered {
        let (state, held, why) = match &view.health {
            Some(health) => (
                health.state.label().to_string(),
                duration_label(health.held_for_seconds(now)),
                health.reason.clone(),
            ),
            // Never blank. A node with no verdict is a node nobody has looked
            // at, which is a different thing from a node that is well.
            None => (
                "not judged".to_string(),
                "-".to_string(),
                "no verdict has been recorded for this node yet".to_string(),
            ),
        };
        let height = view.latest.as_ref().map_or_else(
            || "not checked".to_string(),
            |latest| latest.block_height.cell(|height| height.to_string()),
        );
        lines.push(format!(
            "{:<24} {:<12} {:<10} {:>12}  {}",
            truncate_node_name(&namer(&view.node_id), 24),
            state,
            held,
            height,
            why
        ));
    }

    let attention = count_needing_attention(&views);
    lines.push(String::new());
    lines.push(match attention {
        0 => format!(
            "{} nodes, none needing attention. {} have no verdict yet.",
            views.len(),
            views.iter().filter(|view| view.health.is_none()).count()
        ),
        count => format!("{count} of {} nodes need attention.", views.len()),
    });

    Ok(CliAction::PrintWithExitCode {
        // Non-zero so a gate can fail on it without reading the text.
        exit_code: i32::from(attention > 0),
        text: lines.join("\n"),
    })
}

/// `--fleet-health-json <db>`: the same, for something that has to parse it.
pub(in crate::cli::actions) fn fleet_health_json_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 3, "--fleet-health-json")?;
    let (views, nodes) = load(&args[2])?;
    let namer = |node_id: &str| name_of(&nodes, node_id);
    let now = current_unix_time()?;

    let entries: Vec<serde_json::Value> = views
        .iter()
        .map(|view| json_for(view, &namer(&view.node_id), now))
        .collect();
    let attention = count_needing_attention(&views);
    let payload = serde_json::json!({
        "generated_at_unix": now,
        "nodes": entries,
        "needing_attention": attention,
        "not_judged": views.iter().filter(|view| view.health.is_none()).count(),
    });

    Ok(CliAction::PrintWithExitCode {
        exit_code: i32::from(attention > 0),
        text: serde_json::to_string_pretty(&payload)?,
    })
}

/// `--node-health <db> <node-name>`: one node's verdict, readings and timeline.
pub(in crate::cli::actions) fn node_health_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 4, "--node-health")?;
    let repository = open_workspace(&args[2])?;
    let nodes = repository.list_nodes()?;
    let node = node_by_name(&repository, &args[3])?;
    let now = current_unix_time()?;
    let view = fleet_chain_view(&repository, &nodes, now)?
        .into_iter()
        .find(|view| view.node_id == node.id)
        .ok_or_else(|| anyhow::anyhow!("node {} has no chain view", node.name))?;

    let mut lines = vec![format!("node: {}", node.name), format!("id: {}", node.id)];
    match &view.health {
        Some(health) => {
            lines.push(format!("health: {}", health.state.label()));
            lines.push(format!(
                "held-for: {}",
                duration_label(health.held_for_seconds(now))
            ));
            lines.push(format!(
                "judged: {}s ago",
                health.evaluated_seconds_ago(now)
            ));
            lines.push(format!("why: {}", health.reason));
            if let Some(cause) = &health.cause {
                lines.push(format!("cause: {cause}"));
            }
            if let Some(scope) = health.scope {
                lines.push(format!("affects: {}", scope.label()));
            }
            lines.push(format!("next: {}", next_step_text(&health.next)));
        }
        None => {
            lines.push("health: not judged".to_string());
            lines.push("why: no verdict has been recorded for this node yet".to_string());
        }
    }

    if let Some(latest) = &view.latest {
        lines.push(String::new());
        lines.push(format!("endpoint: {}", blank_as_none(&latest.endpoint)));
        lines.push(format!(
            "block-height: {}",
            latest.block_height.cell(|height| height.to_string())
        ));
        lines.push(format!(
            "peers: {}",
            latest.peers_connected.cell(|peers| peers.to_string())
        ));
        lines.push(format!(
            "rpc-round-trip: {}",
            latest
                .head_latency_ms
                .map_or_else(|| "not measured".to_string(), |ms| format!("{ms} ms"))
        ));
        lines.push(format!(
            "network-magic: {}",
            latest.observed_magic.cell(|magic| magic.to_string())
        ));
        lines.push(format!(
            "client: {}",
            latest.client_version.cell(std::clone::Clone::clone)
        ));
    }

    let timeline = node_health_timeline(&repository, &node.id, TIMELINE_LENGTH)?;
    if !timeline.is_empty() {
        lines.push(String::new());
        lines.push("history:".to_string());
        for transition in &timeline {
            lines.push(format!(
                "  {}s ago  {}",
                now.saturating_sub(transition.at_unix),
                transition.summary()
            ));
        }
    }
    lines.push(String::new());

    let needs_attention = view
        .health
        .as_ref()
        .is_some_and(|health| health.state.needs_attention());
    Ok(CliAction::PrintWithExitCode {
        exit_code: i32::from(needs_attention),
        text: lines.join("\n"),
    })
}

/// `--node-health-json <db> <node-name>`.
pub(in crate::cli::actions) fn node_health_json_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 4, "--node-health-json")?;
    let repository = open_workspace(&args[2])?;
    let nodes = repository.list_nodes()?;
    let node = node_by_name(&repository, &args[3])?;
    let now = current_unix_time()?;
    let view = fleet_chain_view(&repository, &nodes, now)?
        .into_iter()
        .find(|view| view.node_id == node.id)
        .ok_or_else(|| anyhow::anyhow!("node {} has no chain view", node.name))?;
    let timeline = node_health_timeline(&repository, &node.id, TIMELINE_LENGTH)?;

    let mut payload = json_for(&view, &node.name, now);
    payload["history"] = serde_json::Value::Array(
        timeline
            .iter()
            .map(|transition| {
                serde_json::json!({
                    "at_unix": transition.at_unix,
                    "from": transition.from.map(HealthState::persist_key),
                    "to": transition.to.persist_key(),
                    "reason": transition.reason,
                })
            })
            .collect(),
    );

    let needs_attention = view
        .health
        .as_ref()
        .is_some_and(|health| health.state.needs_attention());
    Ok(CliAction::PrintWithExitCode {
        exit_code: i32::from(needs_attention),
        text: serde_json::to_string_pretty(&payload)?,
    })
}

fn load(database: &str) -> Result<(Vec<NodeChainView>, Vec<NodeConfig>)> {
    let repository = open_workspace(database)?;
    let nodes = repository
        .list_nodes()
        .context("failed to read nodes from the workspace")?;
    let now = current_unix_time()?;
    let views = fleet_chain_view(&repository, &nodes, now)?;
    Ok((views, nodes))
}

/// Name a node the way an operator does.
///
/// A node id is a uuid; a report that identifies a node by one names it in the
/// only spelling the operator has never seen. Falls back to the id when the
/// fleet does not contain it, which beats claiming there is no such node.
fn name_of(nodes: &[NodeConfig], node_id: &str) -> String {
    nodes
        .iter()
        .find(|node| node.id == node_id)
        .map_or_else(|| node_id.to_string(), |node| node.name.clone())
}

fn count_needing_attention(views: &[NodeChainView]) -> usize {
    views
        .iter()
        .filter(|view| {
            view.health
                .as_ref()
                .is_some_and(|health| health.state.needs_attention())
        })
        .count()
}

/// One node, as JSON.
///
/// Every measurement is nullable and a null means **not read**, not zero — the
/// same distinction the table keeps. A consumer that treats a null peer count
/// as zero peers has invented an incident.
fn json_for(view: &NodeChainView, name: &str, now: u64) -> serde_json::Value {
    let latest = view.latest.as_ref();
    serde_json::json!({
        "node_id": view.node_id,
        "name": name,
        "health": view.health.as_ref().map(|health| serde_json::json!({
            "state": health.state.persist_key(),
            "needs_attention": health.state.needs_attention(),
            "since_unix": health.since_unix,
            "held_for_seconds": health.held_for_seconds(now),
            "evaluated_at_unix": health.evaluated_at_unix,
            "evaluated_seconds_ago": health.evaluated_seconds_ago(now),
            "reason": health.reason,
            "cause": health.cause,
            "scope": health.scope.map(StallScope::persist_key),
            "next": next_step_text(&health.next),
        })),
        "sampled_at_unix": latest.map(|latest| latest.sampled_at_unix),
        "endpoint": latest.map(|latest| latest.endpoint.clone()),
        "block_height": latest.and_then(|latest| latest.block_height.value().copied()),
        "header_height": latest.and_then(|latest| latest.header_height.value().copied()),
        "peers_connected": latest.and_then(|latest| latest.peers_connected.value().copied()),
        "rpc_round_trip_ms": latest.and_then(|latest| latest.head_latency_ms),
        "observed_magic": latest.and_then(|latest| latest.observed_magic.value().copied()),
        "client_version": latest.and_then(|latest| latest.client_version.value().cloned()),
        "head_lag": view.derived.head_lag,
        "header_gap": view.derived.header_gap,
        "blocks_per_minute": view.derived.blocks_per_minute,
        "chain_lag_seconds": view.derived.chain_lag_seconds,
        "height_unchanged_seconds": view.derived.height_unchanged_seconds,
        "clock_suspect": view.derived.clock_suspect,
    })
}

fn next_step_text(next: &NextStep) -> String {
    match next {
        NextStep::Here { label, href } => format!("{label} ({href})"),
        NextStep::External { text } => text.clone(),
    }
}

/// A node with no RPC port records an empty endpoint; say so rather than print
/// a blank field an operator has to guess at.
fn blank_as_none(endpoint: &str) -> &str {
    if endpoint.is_empty() {
        "none; this node has no RPC port"
    } else {
        endpoint
    }
}
