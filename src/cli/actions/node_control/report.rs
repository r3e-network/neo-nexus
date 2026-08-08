//! Read-only node reporting: the fleet table and the single-node report.
//!
//! Split from the lifecycle actions because these only read. Every read goes
//! through the core facade rather than the repository's row API, so the CLI and
//! the workbench answer the same question the same way.

use super::super::*;
use crate::core::node_health::latest_node_rpc_health;

use super::workspace::{node_by_name, open_workspace};

/// `--node-list <db>`: print every node in the workspace as a compact table, so
/// a script or operator can see fleet status headlessly. Columns are name,
/// type, network, status, rpc port, p2p port.
pub(in crate::cli::actions) fn node_list_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 3, "--node-list")?;
    let repository = open_workspace(&args[2])?;
    let nodes = repository
        .list_nodes()
        .context("failed to read nodes from the workspace")?;

    if nodes.is_empty() {
        return Ok(CliAction::PrintWithExitCode {
            exit_code: 0,
            text: "No nodes in the workspace.".to_string(),
        });
    }

    let mut lines = Vec::with_capacity(nodes.len() + 1);
    lines.push(format!(
        "{:<24} {:<8} {:<8} {:<8} {:>8} {:>8}",
        "NAME", "TYPE", "NETWORK", "STATUS", "RPC", "P2P"
    ));
    for node in &nodes {
        lines.push(format!(
            "{:<24} {:<8} {:<8} {:<8} {:>8} {:>8}",
            truncate_node_name(&node.name, 24),
            node.node_type,
            node.network,
            node.status,
            node.rpc_port,
            node.p2p_port
        ));
    }
    Ok(CliAction::PrintWithExitCode {
        exit_code: 0,
        text: lines.join("\n"),
    })
}

/// `--node-status <db> <node-name>`: print a detailed single-node report
/// (identity, status/pid, ports, version, storage, latest RPC health) so an
/// operator or script can inspect one node headlessly. All reads go through the
/// core facade, never the repository's row API directly.
pub(in crate::cli::actions) fn node_status_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 4, "--node-status")?;
    let repository = open_workspace(&args[2])?;
    let node = node_by_name(&repository, &args[3])?;

    let mut lines = Vec::with_capacity(12);
    lines.push(format!("Name:    {}", node.name));
    lines.push(format!("Type:    {}", node.node_type));
    lines.push(format!("Network: {}", node.network));
    lines.push(format!("Version: {}", node.runtime_version));
    lines.push(format!("Storage: {}", node.storage_engine));
    lines.push(format!("Status:  {}", node.status));
    if let Some(pid) = node.pid {
        lines.push(format!("PID:     {pid}"));
    }
    lines.push(format!("RPC:     {}", node.rpc_port));
    lines.push(format!("P2P:     {}", node.p2p_port));
    if let Some(ws) = node.ws_port {
        lines.push(format!("WS:      {ws}"));
    }
    lines.push(format!("Binary:  {}", node.binary_path.display()));

    // Latest RPC health probe, via the core read operation (not the repository).
    match latest_node_rpc_health(&repository, &node.id) {
        Ok(Some(health)) => {
            lines.push(String::new());
            lines.push("RPC health:".to_string());
            lines.push(format!("  status:   {}", health.status));
            if let Some(height) = health.block_count {
                lines.push(format!("  height:   {height}"));
            }
            lines.push(format!("  endpoint: {}", health.endpoint));
            lines.push(format!("  message:  {}", health.message));
        }
        Ok(None) => lines.push("RPC health: unchecked".to_string()),
        Err(error) => lines.push(format!("RPC health: error — {error}")),
    }

    Ok(CliAction::PrintWithExitCode {
        exit_code: 0,
        text: lines.join("\n"),
    })
}

fn truncate_node_name(name: &str, max: usize) -> String {
    if name.len() <= max {
        name.to_string()
    } else {
        let end = name.char_indices().take(max - 1).last().map(|(i, _)| i);
        if let Some(i) = end {
            format!("{}…", &name[..i])
        } else {
            format!("{}…", name.chars().take(max - 1).collect::<String>())
        }
    }
}
