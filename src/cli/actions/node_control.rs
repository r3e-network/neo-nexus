use std::path::PathBuf;

use super::*;

use crate::core::lifecycle::{execute_node_launch, LaunchAction, ManagedConfig, NodeLaunchOutcome};
use crate::core::node_health::latest_node_rpc_health;
use crate::core::operations::{evaluate_launch_readiness, evaluate_restart_readiness};
use crate::core::workspace::ConfigExporter;
use crate::launch::LaunchPlanner;
use crate::supervisor::{log_path_for, ProcessSupervisor};

/// `--node-start <db> <node-name>`: launch a node through the SAME core pipeline
/// the GUI uses (`execute_node_launch`), so the two modes stay behaviourally
/// identical. Reports readiness blockers before launching and the resulting
/// pid/log path on success.
pub(in crate::cli::actions) fn node_start_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 4, "--node-start")?;
    let repository = open_workspace(&args[2])?;
    let node = node_by_name(&repository, &args[3])?;

    launch_node(
        &repository,
        &node,
        LaunchAction::Start,
        "started",
        "failed to start",
    )
}

/// `--node-restart <db> <node-name>`: restart a node through the SAME core
/// pipeline the GUI uses (evaluate_restart_readiness -> execute_node_launch with
/// Restart), so CLI restart and operator restart behave identically.
pub(in crate::cli::actions) fn node_restart_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 4, "--node-restart")?;
    let repository = open_workspace(&args[2])?;
    let node = node_by_name(&repository, &args[3])?;
    if !node.status.is_running() {
        return Ok(CliAction::PrintWithExitCode {
            exit_code: 1,
            text: format!("{} must be running before restart", node.name),
        });
    }

    launch_node(
        &repository,
        &node,
        LaunchAction::Restart,
        "restarted",
        "failed to restart",
    )
}

/// Shared launch/restart pipeline: evaluate readiness, build the plan, and run
/// `execute_node_launch` with the given action. `verb_past`/`fail_verb` tailor
/// the printed message to start vs restart.
fn launch_node(
    repository: &Repository,
    node: &NodeConfig,
    action: LaunchAction,
    verb_past: &str,
    fail_verb: &str,
) -> Result<CliAction> {
    let plugins = repository
        .list_plugin_states(&node.id)
        .context("failed to read plugin states")?;
    let work_dir = workspace_child_dir(repository, "nodes").join(&node.id);
    let managed_config_path = ConfigExporter::managed_target_path(&work_dir, node);
    let log_path = log_path_for(workspace_child_dir(repository, "logs"), node);

    let readiness = match action {
        LaunchAction::Start => evaluate_launch_readiness(
            node,
            std::slice::from_ref(node),
            &plugins,
            &managed_config_path,
            &work_dir,
        ),
        LaunchAction::Restart => evaluate_restart_readiness(
            node,
            std::slice::from_ref(node),
            &plugins,
            &managed_config_path,
            &work_dir,
        ),
    };
    if let Some(blocker) = readiness.blocking_summary() {
        return Ok(CliAction::PrintWithExitCode {
            exit_code: 1,
            text: format!(
                "{} not {verb_past}: readiness blocked — {blocker}",
                node.name
            ),
        });
    }

    let plan = LaunchPlanner::plan(node, &managed_config_path, &work_dir);
    let mut supervisor = ProcessSupervisor::default();
    let outcome = execute_node_launch(
        repository,
        &mut supervisor,
        node,
        &plan,
        &log_path,
        action,
        Some(ManagedConfig {
            path: &managed_config_path,
            plugins: &plugins,
        }),
    );

    Ok(match outcome {
        NodeLaunchOutcome::Started { pid, log_path } => CliAction::PrintWithExitCode {
            exit_code: 0,
            text: format!(
                "{} {verb_past} with PID {}; log {}",
                node.name,
                pid,
                log_path.display()
            ),
        },
        NodeLaunchOutcome::Failed { message } => CliAction::PrintWithExitCode {
            exit_code: 1,
            text: format!("{} {fail_verb}: {message}", node.name),
        },
    })
}

/// `--node-stop <db> <node-name>`: stop a node via the shared supervisor, then
/// persist the stopped status, mirroring the GUI's stop path.
pub(in crate::cli::actions) fn node_stop_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 4, "--node-stop")?;
    let repository = open_workspace(&args[2])?;
    let node = node_by_name(&repository, &args[3])?;

    let mut supervisor = ProcessSupervisor::default();
    let stopped = supervisor
        .stop(&node.id)
        .context("failed to stop the supervised process")?;

    repository
        .update_node_status(&node.id, NodeStatus::Stopped, None)
        .context("failed to persist stopped status")?;

    Ok(CliAction::PrintWithExitCode {
        exit_code: 0,
        text: match stopped {
            Some(_) => format!("{} stopped", node.name),
            None => format!("{} was not running", node.name),
        },
    })
}

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

fn open_workspace(db_path: &str) -> Result<Repository> {
    Repository::open(PathBuf::from(db_path))
        .with_context(|| format!("failed to open workspace database {db_path}"))
}

fn node_by_name(repository: &Repository, name: &str) -> Result<NodeConfig> {
    let nodes = repository
        .list_nodes()
        .context("failed to read nodes from the workspace")?;
    nodes
        .into_iter()
        .find(|node| node.name == name)
        .with_context(|| format!("no node named {name:?} in the workspace"))
}

/// Mirrors NeoNexusApp::workspace_child_dir: a subdirectory beside the database,
/// so the CLI writes managed configs and logs to the same place the GUI would.
fn workspace_child_dir(repository: &Repository, child: &str) -> PathBuf {
    repository
        .db_path()
        .parent()
        .map_or_else(|| PathBuf::from(child), |parent| parent.join(child))
}
