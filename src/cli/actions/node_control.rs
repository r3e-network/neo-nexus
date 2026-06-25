use std::path::PathBuf;

use super::*;

use crate::core::lifecycle::{execute_node_launch, LaunchAction, ManagedConfig, NodeLaunchOutcome};
use crate::core::operations::evaluate_launch_readiness;
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
    let plugins = repository
        .list_plugin_states(&node.id)
        .context("failed to read plugin states")?;
    let work_dir = workspace_child_dir(&repository, "nodes").join(&node.id);
    let managed_config_path = ConfigExporter::managed_target_path(&work_dir, &node);
    let log_path = log_path_for(workspace_child_dir(&repository, "logs"), &node);

    let readiness = evaluate_launch_readiness(
        &node,
        std::slice::from_ref(&node),
        &plugins,
        &managed_config_path,
        &work_dir,
    );
    if let Some(blocker) = readiness.blocking_summary() {
        return Ok(CliAction::PrintWithExitCode {
            exit_code: 1,
            text: format!(
                "{} not started: launch readiness blocked — {blocker}",
                node.name
            ),
        });
    }

    let plan = LaunchPlanner::plan(&node, &managed_config_path, &work_dir);
    let mut supervisor = ProcessSupervisor::default();
    let outcome = execute_node_launch(
        &repository,
        &mut supervisor,
        &node,
        &plan,
        &log_path,
        LaunchAction::Start,
        Some(ManagedConfig {
            path: &managed_config_path,
            plugins: &plugins,
        }),
    );

    Ok(match outcome {
        NodeLaunchOutcome::Started { pid, log_path } => CliAction::PrintWithExitCode {
            exit_code: 0,
            text: format!(
                "{} started with PID {}; log {}",
                node.name,
                pid,
                log_path.display()
            ),
        },
        NodeLaunchOutcome::Failed { message } => CliAction::PrintWithExitCode {
            exit_code: 1,
            text: format!("{} failed to start: {message}", node.name),
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
