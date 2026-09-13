//! Node lifecycle orchestration: the shared core that wires config export,
//! process supervision, and status persistence into one operation usable by both
//! the GUI shell and a headless CLI.
//!
//! Previously this pipeline (export managed config -> supervisor.start/restart ->
//! `repository.update_node_status`) was duplicated three times inside `src/app/`
//! (`node_lifecycle_flow/runtime/{start,restart}.rs` and
//! `managed_config_flow/config_actions/restart.rs`). Each copy re-implemented the
//! failure -> `update_node_status(Error)` path, and none existed in core — so the
//! "extract core, support dual mode" goal was unmet for the single most important
//! operation.
//!
//! This module is the one source of truth. It knows nothing about UI shells, notices,
//! or event journals — those are presentation concerns that each frontend maps the
//! [`NodeLaunchOutcome`] to. The readiness evaluation stays a caller
//! responsibility (it returns a rich report the frontend may surface), so a future
//! CLI `--node-start` and the GUI use the identical launch path.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::{
    catalog::PluginState,
    config::ConfigExporter,
    core::node_signer::prepare_node_signer_launch,
    launch::LaunchPlan,
    repository::Repository,
    signing::SignerRegistry,
    supervisor::{PidStop, ProcessStart, ProcessSupervisor, LAUNCH_SETTLE_WINDOW},
    types::{NodeConfig, NodeStatus},
};

/// Whether to start a fresh process or restart a running one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchAction {
    /// Spawn the node process fresh (it is not currently running).
    Start,
    /// Stop then start again (used for restart and managed-config-apply).
    Restart,
}

/// The outcome of executing a node launch or restart. The core persists status on
/// every path (success -> `Running` + pid; failure -> `Error`), so the frontend
/// only needs to map the outcome to its own notice/event presentation.
#[derive(Debug, Clone)]
pub enum NodeLaunchOutcome {
    /// The process launched and its `Running` status + pid were persisted.
    Started {
        pid: u32,
        log_path: PathBuf,
        replaced_unmanaged: bool,
    },
    /// A step failed. When the failure happened at supervision time, the node's
    /// status has already been persisted as `Error`; for an export failure the
    /// prior status is left untouched.
    Failed { message: String },
}

/// Immutable inputs for one launch attempt. Grouping them makes the lifecycle
/// boundary explicit while the repository and supervisor remain shared
/// orchestration services.
#[derive(Debug)]
pub struct NodeLaunchRequest<'a> {
    pub signer_registry: Option<&'a SignerRegistry>,
    pub node: &'a NodeConfig,
    pub plan: &'a LaunchPlan,
    pub log_path: PathBuf,
    pub action: LaunchAction,
    pub managed_config: Option<ManagedConfig<'a>>,
}

/// Executes the shared launch/restart pipeline: optionally exports the managed
/// config, runs (or restarts) the supervised process, and persists the resulting
/// status. Returns a [`NodeLaunchOutcome`] the caller maps to notices/events.
///
/// `managed_config_path`, when `Some`, is written via [`ConfigExporter`] before
/// supervision; `None` means the node uses runtime args only and there is nothing
/// to export (e.g. a restart that already has its config on disk).
pub fn execute_node_launch(
    repository: &Repository,
    supervisor: &mut ProcessSupervisor,
    request: NodeLaunchRequest<'_>,
) -> NodeLaunchOutcome {
    let NodeLaunchRequest {
        signer_registry,
        node,
        plan,
        log_path,
        action,
        managed_config,
    } = request;
    if action == LaunchAction::Start && (node.status.is_active() || node.pid.is_some()) {
        return NodeLaunchOutcome::Failed {
            message: format!(
                "{} is active or still has a recorded process; stop and settle it before starting",
                node.name
            ),
        };
    }
    if action == LaunchAction::Restart
        && (!node.status.is_running() || (node.pid.is_none() && !supervisor.is_managing(&node.id)))
    {
        return NodeLaunchOutcome::Failed {
            message: format!(
                "{} must have a running process controlled by a handle or recorded pid before restart",
                node.name
            ),
        };
    }

    let signer_plugins = managed_config.as_ref().map_or_else(
        || repository.list_plugin_states(&node.id),
        |config| Ok(config.plugins.to_vec()),
    );
    let signer_plugins = match signer_plugins {
        Ok(plugins) => plugins,
        Err(error) => {
            return NodeLaunchOutcome::Failed {
                message: format!("failed to read {} plugin state: {error}", node.name),
            }
        }
    };
    // One typed decision drives config and process startup. A missing profile,
    // incompatible runtime, wrong network/key, or misplaced plugin fails here
    // before the launch lease is claimed.
    let signer_launch = match prepare_node_signer_launch(
        repository,
        signer_registry,
        node,
        &signer_plugins,
        &plan.working_dir,
    ) {
        Ok(launch) => launch,
        Err(error) => {
            return NodeLaunchOutcome::Failed {
                message: error.to_string(),
            }
        }
    };

    // `Starting` is the operation lease shared by every controller. Keep the
    // old pid during a restart: if this process crashes while rendering config,
    // a later Stop can still reach the untouched old process.
    let claim_pid = node.pid;
    let claimed = repository.claim_node_launch(node);
    match claimed {
        Ok(true) => {}
        Ok(false) => {
            return NodeLaunchOutcome::Failed {
                message: format!(
                    "{} runtime state changed concurrently; reload it before launching",
                    node.name
                ),
            };
        }
        Err(error) => {
            return NodeLaunchOutcome::Failed {
                message: format!("failed to claim {} for launch: {error}", node.name),
            };
        }
    }

    if let Some(config) = managed_config {
        // Rendered for the duty the workspace records, not as a bare relay. A
        // context-free render here silently overwrote the section an operator
        // had just applied, so the node started as a relay while the workbench
        // still showed its duty.
        let generation = signer_launch.generation_context();
        if let Err(error) = ConfigExporter::write_node_config_to_path_with_context(
            config.path,
            node,
            config.plugins,
            None,
            &generation,
        ) {
            restore_launch_claim(repository, node, claim_pid);
            return NodeLaunchOutcome::Failed {
                message: error.to_string(),
            };
        }
    }

    // Configuration is known-good before a restart stops the healthy old
    // process. Failure to identify or stop it restores the original state.
    let replaced_unmanaged = if action == LaunchAction::Restart {
        match quiesce_before_restart(supervisor, node, &log_path) {
            Ok(replaced) => replaced,
            Err(error) => {
                restore_launch_claim(repository, node, claim_pid);
                return NodeLaunchOutcome::Failed {
                    message: error.to_string(),
                };
            }
        }
    } else {
        false
    };

    // A concurrent Stop may have claimed the row while config was rendered or
    // the old process was quiesced. Revalidate the lease before spawning.
    let still_claimed = repository.transition_node_status(
        &node.id,
        NodeStatus::Starting,
        claim_pid,
        NodeStatus::Starting,
        claim_pid,
    );
    match still_claimed {
        Ok(true) => {}
        Ok(false) => {
            return NodeLaunchOutcome::Failed {
                message: format!(
                    "{} runtime state changed concurrently; reload it before launching",
                    node.name
                ),
            };
        }
        Err(error) => {
            return NodeLaunchOutcome::Failed {
                message: format!("failed to claim {} for launch: {error}", node.name),
            };
        }
    }

    let start = match action {
        LaunchAction::Start => supervisor.start(node, plan, &log_path),
        LaunchAction::Restart => supervisor.restart(node, plan, &log_path),
    };

    match start {
        Ok(ProcessStart { pid, log_path }) => {
            // `spawn` returning only proves the kernel accepted the executable.
            // Watch the process long enough to notice one that dies of its own
            // arguments, so the row never says Running for something that is
            // already gone and the operator gets the runtime's own reason
            // instead of a pid to go hunting for.
            if let Some(reason) = supervisor
                .confirm_startup(&node.id, LAUNCH_SETTLE_WINDOW)
                .failure_summary(&node.name)
            {
                let _ = repository.transition_node_status(
                    &node.id,
                    NodeStatus::Starting,
                    claim_pid,
                    NodeStatus::Error,
                    None,
                );
                return NodeLaunchOutcome::Failed { message: reason };
            }
            let persisted = repository.transition_node_status(
                &node.id,
                NodeStatus::Starting,
                claim_pid,
                NodeStatus::Running,
                Some(pid),
            );
            if !matches!(persisted, Ok(true)) {
                let cleanup =
                    cleanup_uncommitted_start(repository, supervisor, node, claim_pid, pid);
                let persistence = match persisted {
                    Ok(false) => "runtime state changed concurrently".to_string(),
                    Err(error) => error.to_string(),
                    Ok(true) => unreachable!(),
                };
                return NodeLaunchOutcome::Failed {
                    message: format!(
                        "failed to persist {} as running: {persistence}; {cleanup}",
                        node.name
                    ),
                };
            }
            NodeLaunchOutcome::Started {
                pid,
                log_path,
                replaced_unmanaged,
            }
        }
        Err(error) => {
            // Do not overwrite a concurrent Stop. When a managed restart failed
            // while stopping its old child, retain the old pid so the operator
            // still has a durable handle for a retry.
            let retained_pid = supervisor
                .is_managing(&node.id)
                .then_some(node.pid)
                .flatten();
            let _ = repository.transition_node_status(
                &node.id,
                NodeStatus::Starting,
                claim_pid,
                NodeStatus::Error,
                retained_pid,
            );
            NodeLaunchOutcome::Failed {
                message: error.to_string(),
            }
        }
    }
}

pub fn quiesce_before_restart(
    supervisor: &mut ProcessSupervisor,
    node: &NodeConfig,
    log_path: impl AsRef<Path>,
) -> Result<bool> {
    if supervisor.is_managing(&node.id) {
        // `restart` will stop and replace it through the handle it owns.
        return Ok(false);
    }
    if node.pid.is_none() {
        return Ok(false);
    }
    match supervisor.stop_recorded_pid(node, log_path) {
        PidStop::Stopped(_) => Ok(true),
        PidStop::AlreadyGone => Ok(false),
        PidStop::PidReused => anyhow::bail!(
            "pid {} belongs to another process; refusing to restart {} on top of it",
            node.pid.unwrap_or_default(),
            node.name
        ),
        PidStop::Failed { message, .. } => {
            anyhow::bail!("could not quiesce {} before restart: {message}", node.name)
        }
    }
}

/// Persist a stop request before touching the process, keep its pid as the
/// recovery handle while termination is in flight, and clear that pid only
/// after the process is confirmed gone. CLI and Web both use this protocol so
/// the watchdog observes the same durable stop intent.
pub fn stop_node_runtime(
    repository: &Repository,
    supervisor: &mut ProcessSupervisor,
    node: &NodeConfig,
    log_path: impl AsRef<Path>,
) -> Result<PidStop> {
    let target = claim_stop_intent(repository, &node.id)?;
    let outcome = match supervisor.stop(&target.id) {
        Ok(Some(stop)) => PidStop::Stopped(stop),
        Ok(None) => supervisor.stop_recorded_pid(&target, log_path),
        Err(error) => {
            restore_after_failed_stop(repository, &target)?;
            return Err(error).context("failed to stop the supervised process");
        }
    };

    match outcome {
        PidStop::Stopped(_) | PidStop::AlreadyGone => {
            settle_stopped_node(repository, &target)?;
        }
        PidStop::PidReused | PidStop::Failed { .. } => {
            restore_after_failed_stop(repository, &target)?;
        }
    }
    Ok(outcome)
}

fn claim_stop_intent(repository: &Repository, node_id: &str) -> Result<NodeConfig> {
    for _ in 0..3 {
        let current = load_node(repository, node_id)?;
        if repository.transition_node_status(
            node_id,
            current.status,
            current.pid,
            NodeStatus::Stopped,
            current.pid,
        )? {
            return Ok(current);
        }
    }
    anyhow::bail!("node {node_id} runtime state kept changing; stop was not attempted")
}

fn settle_stopped_node(repository: &Repository, node: &NodeConfig) -> Result<()> {
    if repository.transition_node_status(
        &node.id,
        NodeStatus::Stopped,
        node.pid,
        NodeStatus::Stopped,
        None,
    )? {
        return Ok(());
    }
    let current = load_node(repository, &node.id)?;
    if current.status == NodeStatus::Stopped && current.pid.is_none() {
        return Ok(());
    }
    anyhow::bail!(
        "{} stopped, but its runtime state changed before the pid could be settled",
        node.name
    )
}

fn restore_after_failed_stop(repository: &Repository, node: &NodeConfig) -> Result<()> {
    if repository.transition_node_status(
        &node.id,
        NodeStatus::Stopped,
        node.pid,
        node.status,
        node.pid,
    )? {
        return Ok(());
    }
    anyhow::bail!(
        "stop of {} failed and its concurrently changed runtime state was left untouched",
        node.name
    )
}

fn load_node(repository: &Repository, node_id: &str) -> Result<NodeConfig> {
    repository
        .list_nodes()?
        .into_iter()
        .find(|node| node.id == node_id)
        .with_context(|| format!("node {node_id} was not found"))
}

fn cleanup_uncommitted_start(
    repository: &Repository,
    supervisor: &mut ProcessSupervisor,
    node: &NodeConfig,
    claim_pid: Option<u32>,
    pid: u32,
) -> String {
    match supervisor.stop(&node.id) {
        Ok(Some(_)) => {
            let _ = repository.transition_node_status(
                &node.id,
                NodeStatus::Starting,
                claim_pid,
                NodeStatus::Error,
                None,
            );
            "the uncommitted process was stopped".to_string()
        }
        Ok(None) => {
            preserve_unsettled_pid(repository, node, claim_pid, pid);
            format!("no process handle remained for pid {pid}")
        }
        Err(error) => {
            preserve_unsettled_pid(repository, node, claim_pid, pid);
            format!("cleanup of pid {pid} failed: {error}")
        }
    }
}

fn preserve_unsettled_pid(
    repository: &Repository,
    node: &NodeConfig,
    claim_pid: Option<u32>,
    pid: u32,
) {
    let _ = repository.transition_node_status(
        &node.id,
        NodeStatus::Starting,
        claim_pid,
        NodeStatus::Error,
        Some(pid),
    );
    // A concurrent Stop may have won between spawn and persistence. Never
    // overwrite that durable intent; the retained supervisor handle is still
    // reachable by the waiting Web stop (or by this supervisor's Drop in CLI).
}

fn restore_launch_claim(repository: &Repository, node: &NodeConfig, claim_pid: Option<u32>) {
    let _ = repository.transition_node_status(
        &node.id,
        NodeStatus::Starting,
        claim_pid,
        node.status,
        node.pid,
    );
}

/// The managed config to write before launching, with the plugins needed to
/// render it. Passed as a struct so the export step and its inputs stay together
/// at the call site.
#[derive(Debug, Clone, Copy)]
pub struct ManagedConfig<'a> {
    pub path: &'a Path,
    pub plugins: &'a [PluginState],
}
