//! Node lifecycle from the command line: start, restart, stop.
//!
//! Every one runs the same core pipeline the workbench runs, so a scripted
//! node and an operator-driven node behave identically. Reporting lives in
//! `report`, and workspace lookup in `workspace`.

mod report;
mod workspace;

pub(in crate::cli::actions) use report::{node_list_action, node_status_action};
use workspace::{node_by_name, open_workspace, workspace_child_dir};

use super::*;

use crate::core::lifecycle::{
    execute_node_launch, stop_node_runtime, LaunchAction, ManagedConfig, NodeLaunchOutcome,
    NodeLaunchRequest,
};
use crate::core::node::node_workspace_path;
use crate::core::node_signer::{node_signer_key, resolve_node_signer};
use crate::core::operations::{evaluate_launch_readiness, evaluate_restart_readiness};
use crate::core::workspace::ConfigExporter;
use crate::launch::LaunchPlanner;
use crate::supervisor::{log_path_for, PidStop, ProcessSupervisor};

/// `--node-rebind-runtime <db> <node-name> <binary> [runtime-args...]`:
/// replace backup-supplied launch material with a deliberate local choice.
pub(in crate::cli::actions) fn node_rebind_runtime_action(args: &[String]) -> Result<CliAction> {
    if args.len() < 5 {
        anyhow::bail!(
            "--node-rebind-runtime is missing required arguments; run neo-nexus --help for usage"
        );
    }
    let repository = open_workspace(&args[2])?;
    let node = node_by_name(&repository, &args[3])?;
    let rebound = repository
        .rebind_node_runtime(&node.id, PathBuf::from(&args[4]), args[5..].to_vec())
        .context("failed to bind the trusted local node runtime")?;
    Ok(CliAction::PrintWithExitCode {
        exit_code: 0,
        text: format!(
            "{} runtime rebound to {}; backup launch quarantine cleared",
            rebound.name,
            rebound.binary_path.display()
        ),
    })
}

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
    if node.binary_path.as_os_str().is_empty() {
        return Ok(CliAction::PrintWithExitCode {
            exit_code: 1,
            text: format!(
                "{} not restarted: no trusted local runtime is bound; run --node-rebind-runtime first",
                node.name
            ),
        });
    }
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
    let signer_registry = if node_signer_key(repository, node)?.is_some() {
        let registry = crate::signing::SignerRegistry::from_process_environment()
            .context("failed to load the signer registry for this node")?;
        resolve_node_signer(repository, &registry, node)?;
        Some(registry)
    } else {
        None
    };
    let plugins = repository
        .list_plugin_states(&node.id)
        .context("failed to read plugin states")?;
    let all_nodes = repository
        .list_nodes()
        .context("failed to read the node inventory")?;
    let work_dir = node_workspace_path(workspace_child_dir(repository, "nodes"), &node.id)?;
    let managed_config_path = ConfigExporter::managed_target_path(&work_dir, node);
    let log_path = log_path_for(workspace_child_dir(repository, "logs"), node);

    let readiness = match action {
        LaunchAction::Start => {
            evaluate_launch_readiness(node, &all_nodes, &plugins, &managed_config_path, &work_dir)
        }
        LaunchAction::Restart => {
            evaluate_restart_readiness(node, &all_nodes, &plugins, &managed_config_path, &work_dir)
        }
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
        NodeLaunchRequest {
            signer_registry: signer_registry.as_ref(),
            node,
            plan: &plan,
            log_path,
            action,
            managed_config: plan
                .managed_config_path
                .as_deref()
                .map(|path| ManagedConfig {
                    path,
                    plugins: &plugins,
                }),
        },
    );

    // A one-shot command cannot supervise: `ProcessSupervisor` terminates
    // everything still registered when it drops, which meant this reported the
    // node as started and then killed it on the way out of `main`. Hand the
    // process over instead of quietly dropping it; `--node-stop` reaches it by
    // pid, since no handle survives this process.
    if matches!(outcome, NodeLaunchOutcome::Started { .. }) {
        supervisor.disown_all();
    }

    Ok(match outcome {
        NodeLaunchOutcome::Started { pid, log_path, .. } => CliAction::PrintWithExitCode {
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

/// `--node-stop <db> <node-name>`: stop a node and persist the stopped status.
///
/// A one-shot command cannot hold the handle of a process started by an earlier
/// invocation, or by the workbench, so the recorded pid is the fallback —
/// without it this reported "was not running" while the node kept running.
pub(in crate::cli::actions) fn node_stop_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 4, "--node-stop")?;
    let repository = open_workspace(&args[2])?;
    let node = node_by_name(&repository, &args[3])?;

    let log_path = log_path_for(workspace_child_dir(&repository, "logs"), &node);
    let mut supervisor = ProcessSupervisor::default();
    let outcome = stop_node_runtime(&repository, &mut supervisor, &node, &log_path)?;
    let _ = supervisor;
    Ok(CliAction::PrintWithExitCode {
        exit_code: match outcome {
            PidStop::PidReused | PidStop::Failed { .. } => 1,
            _ => 0,
        },
        text: match outcome {
            PidStop::Stopped(stop) if stop.forced => {
                format!("{} stopped (forced, pid {})", node.name, stop.pid)
            }
            PidStop::Stopped(stop) => format!("{} stopped (pid {})", node.name, stop.pid),
            PidStop::AlreadyGone => format!("{} was not running", node.name),
            PidStop::PidReused => format!(
                "pid {} belongs to a different process; {} was left alone and its status unchanged",
                node.pid.unwrap_or_default(),
                node.name
            ),
            PidStop::Failed { message, .. } => {
                format!("{} was not stopped: {message}", node.name)
            }
        },
    })
}
