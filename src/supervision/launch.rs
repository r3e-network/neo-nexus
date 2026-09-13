//! Launching and stopping a node through the shared pipeline.
//!
//! Both directions go through the same readiness checks and the same supervisor
//! as the watchdog's own restarts, so an automatic restart cannot drift from a
//! manual one.

use crate::{
    config::ConfigExporter,
    core::{
        lifecycle::{execute_node_launch, stop_node_runtime, LaunchAction, NodeLaunchRequest},
        node::NodeConfig,
        node_signer::resolve_node_signer,
        operations::{evaluate_launch_readiness, evaluate_restart_readiness},
    },
    events::{EventKind, EventSeverity},
    launch::LaunchPlanner,
    node_lifecycle::ManagedConfig,
    supervisor::{log_path_for, PidStop},
    types::node_workspace_path,
};

use super::state::EngineState;

/// Launch or restart a node through the shared pipeline: readiness first, then
/// managed config, then supervision, then status. Used by the browser and by
/// the watchdog, so an automatic restart cannot drift from a manual one.
pub fn launch_node(
    state: &EngineState,
    node: &NodeConfig,
    action: LaunchAction,
) -> anyhow::Result<String> {
    resolve_node_signer(&state.repository, &state.signer_registry, node)?;
    let plugins = state.repository.list_plugin_states(&node.id)?;
    let all_nodes = state.repository.list_nodes()?;
    let work_dir = node_workspace_path(state.workspace_child_dir("nodes"), &node.id)?;
    let managed_config_path = ConfigExporter::managed_target_path(&work_dir, node);
    let log_path = log_path_for(state.workspace_child_dir("logs"), node);

    let readiness = match action {
        LaunchAction::Start => {
            evaluate_launch_readiness(node, &all_nodes, &plugins, &managed_config_path, &work_dir)
        }
        LaunchAction::Restart => {
            evaluate_restart_readiness(node, &all_nodes, &plugins, &managed_config_path, &work_dir)
        }
    };
    if let Some(blocker) = readiness.blocking_summary() {
        anyhow::bail!("readiness blocked — {blocker}");
    }

    let plan = LaunchPlanner::plan(node, &managed_config_path, &work_dir);
    let mut supervisor = state.supervisor();
    let outcome = execute_node_launch(
        &state.repository,
        &mut supervisor,
        NodeLaunchRequest {
            signer_registry: Some(&state.signer_registry),
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
    drop(supervisor);

    match outcome {
        crate::core::lifecycle::NodeLaunchOutcome::Started {
            pid,
            log_path,
            replaced_unmanaged,
        } => {
            let message = format!(
                "{}{} launched with PID {}; log {}",
                if replaced_unmanaged {
                    "replaced an unmanaged process; "
                } else {
                    ""
                },
                node.name,
                pid,
                log_path.display()
            );
            // The journal records which control ran, so an operator asking
            // "who started this at 03:00" gets one answer rather than a gap.
            // A watchdog restart also lands here and adds its own entry above
            // this one, so the trigger and the effect are both visible.
            state.journal(
                node,
                match action {
                    LaunchAction::Start => EventKind::NodeStarted,
                    LaunchAction::Restart => EventKind::NodeRestarted,
                },
                EventSeverity::Info,
                message.clone(),
            );
            Ok(message)
        }
        crate::core::lifecycle::NodeLaunchOutcome::Failed { message } => {
            anyhow::bail!("{message}")
        }
    }
}

/// Stop a node, reaching the process by pid when this server holds no handle for
/// it. Marks the row stopped only after the process is confirmed gone or was
/// already absent.
pub fn stop_node(state: &EngineState, node: &NodeConfig) -> anyhow::Result<String> {
    let log_path = log_path_for(state.workspace_child_dir("logs"), node);
    let outcome = {
        let mut supervisor = state.supervisor();
        stop_node_runtime(&state.repository, &mut supervisor, node, &log_path)?
    };
    match outcome {
        PidStop::Stopped(stop) => {
            let message = if stop.forced {
                format!("{} stopped (forced, pid {})", node.name, stop.pid)
            } else {
                format!("{} stopped (pid {})", node.name, stop.pid)
            };
            state.journal(
                node,
                EventKind::NodeStopped,
                EventSeverity::Info,
                message.clone(),
            );
            Ok(message)
        }
        PidStop::AlreadyGone => Ok(format!("{} was not running", node.name)),
        // The number is held by something else now. We cannot know whether this
        // node is running, so nothing is signalled and no status is written.
        PidStop::PidReused => Err(anyhow::anyhow!(
            "pid {} belongs to a different process; {name} was left alone and its status unchanged",
            node.pid.unwrap_or_default(),
            name = node.name
        )),
        PidStop::Failed { message, .. } => {
            Err(anyhow::anyhow!("{} was not stopped: {message}", node.name))
        }
    }
}
