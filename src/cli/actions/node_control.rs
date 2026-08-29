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

use crate::core::lifecycle::{execute_node_launch, LaunchAction, ManagedConfig, NodeLaunchOutcome};
use crate::core::operations::{evaluate_launch_readiness, evaluate_restart_readiness};
use crate::core::workspace::ConfigExporter;
use crate::launch::LaunchPlanner;
use crate::supervisor::{log_path_for, PidStop, ProcessSupervisor};

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

    // A one-shot command cannot supervise: `ProcessSupervisor` terminates
    // everything still registered when it drops, which meant this reported the
    // node as started and then killed it on the way out of `main`. Hand the
    // process over instead of quietly dropping it; `--node-stop` reaches it by
    // pid, since no handle survives this process.
    if matches!(outcome, NodeLaunchOutcome::Started { .. }) {
        supervisor.disown_all();
    }

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
    let outcome = match supervisor
        .stop(&node.id)
        .context("failed to stop the supervised process")?
    {
        Some(stop) => PidStop::Stopped(stop),
        None => supervisor.stop_recorded_pid(&node, &log_path),
    };
    if matches!(outcome, PidStop::PidReused) {
        // The number belongs to something else now: nothing was signalled and
        // the recorded status stays as it was, because we cannot know.
        return Ok(CliAction::PrintWithExitCode {
            exit_code: 1,
            text: format!(
                "pid {} belongs to a different process; {} was left alone and its status unchanged",
                node.pid.unwrap_or_default(),
                node.name
            ),
        });
    }
    repository
        .update_node_status(&node.id, NodeStatus::Stopped, None)
        .context("failed to persist stopped status")?;
    let _ = supervisor;
    Ok(CliAction::PrintWithExitCode {
        exit_code: 0,
        text: match outcome {
            PidStop::Stopped(stop) if stop.forced => {
                format!("{} stopped (forced, pid {})", node.name, stop.pid)
            }
            PidStop::Stopped(stop) => format!("{} stopped (pid {})", node.name, stop.pid),
            _ => format!("{} was not running", node.name),
        },
    })
}
