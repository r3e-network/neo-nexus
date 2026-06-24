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
//! This module is the one source of truth. It knows nothing about egui, notices,
//! or event journals — those are presentation concerns that each frontend maps the
//! [`NodeLaunchOutcome`] to. The readiness evaluation stays a caller
//! responsibility (it returns a rich report the frontend may surface), so a future
//! CLI `--node-start` and the GUI use the identical launch path.

use std::path::{Path, PathBuf};

use crate::{
    catalog::PluginState,
    config::ConfigExporter,
    launch::LaunchPlan,
    repository::Repository,
    supervisor::{ProcessStart, ProcessSupervisor},
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
    Started { pid: u32, log_path: PathBuf },
    /// A step failed. When the failure happened at supervision time, the node's
    /// status has already been persisted as `Error`; for an export failure the
    /// prior status is left untouched.
    Failed { message: String },
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
    node: &NodeConfig,
    plan: &LaunchPlan,
    log_path: impl AsRef<Path>,
    action: LaunchAction,
    managed_config: Option<ManagedConfig<'_>>,
) -> NodeLaunchOutcome {
    if let Some(config) = managed_config {
        if let Err(error) =
            ConfigExporter::write_node_config_to_path(config.path, node, config.plugins)
        {
            return NodeLaunchOutcome::Failed {
                message: error.to_string(),
            };
        }
    }

    let start = match action {
        LaunchAction::Start => supervisor.start(node, plan, log_path.as_ref()),
        LaunchAction::Restart => supervisor.restart(node, plan, log_path.as_ref()),
    };

    match start {
        Ok(ProcessStart { pid, log_path }) => {
            if let Err(error) =
                repository.update_node_status(&node.id, NodeStatus::Running, Some(pid))
            {
                return NodeLaunchOutcome::Failed {
                    message: error.to_string(),
                };
            }
            NodeLaunchOutcome::Started { pid, log_path }
        }
        Err(error) => {
            // Persist the error status so the workspace reflects the failed
            // attempt; mirror the behaviour the GUI copies implemented inline.
            let _ = repository.update_node_status(&node.id, NodeStatus::Error, None);
            NodeLaunchOutcome::Failed {
                message: error.to_string(),
            }
        }
    }
}

/// The managed config to write before launching, with the plugins needed to
/// render it. Passed as a struct so the export step and its inputs stay together
/// at the call site.
#[derive(Debug, Clone, Copy)]
pub struct ManagedConfig<'a> {
    pub path: &'a Path,
    pub plugins: &'a [PluginState],
}
