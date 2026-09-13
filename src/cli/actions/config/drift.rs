use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::super::*;
use crate::core::node::NodeConfig;
use crate::core::workspace::{ConfigDriftDetector, ConfigReconciler, Repository};

pub(in crate::cli::actions) fn check_config_drift_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 5, "--check-config-drift")?;
    let repository = open_workspace(&args[2])?;
    let node = find_node_by_name(&repository, &args[3])?;
    let report = ConfigDriftDetector::check(&node, Path::new(&args[4]))?;

    Ok(CliAction::PrintWithExitCode {
        exit_code: report.exit_code(),
        text: report.to_cli_text(),
    })
}

pub(in crate::cli::actions) fn check_config_drift_json_action(
    args: &[String],
) -> Result<CliAction> {
    require_arg_count(args, 5, "--check-config-drift-json")?;
    let repository = open_workspace(&args[2])?;
    let node = find_node_by_name(&repository, &args[3])?;
    let report = ConfigDriftDetector::check(&node, Path::new(&args[4]))?;

    Ok(CliAction::PrintWithExitCode {
        exit_code: report.exit_code(),
        text: serde_json::to_string_pretty(&report)?,
    })
}

pub(in crate::cli::actions) fn reconcile_node_config_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 5, "--reconcile-node-config")?;
    let repository = open_workspace(&args[2])?;
    let node = find_node_by_name(&repository, &args[3])?;
    let report = ConfigReconciler::reconcile(&node, Path::new(&args[4]), true)?;

    let exit_code = if report.post_check_status.is_in_sync() {
        0
    } else {
        1
    };

    Ok(CliAction::PrintWithExitCode {
        exit_code,
        text: report.to_cli_text(),
    })
}

pub(in crate::cli::actions) fn reconcile_node_config_json_action(
    args: &[String],
) -> Result<CliAction> {
    require_arg_count(args, 5, "--reconcile-node-config-json")?;
    let repository = open_workspace(&args[2])?;
    let node = find_node_by_name(&repository, &args[3])?;
    let report = ConfigReconciler::reconcile(&node, Path::new(&args[4]), true)?;

    let exit_code = if report.post_check_status.is_in_sync() {
        0
    } else {
        1
    };

    Ok(CliAction::PrintWithExitCode {
        exit_code,
        text: serde_json::to_string_pretty(&report)?,
    })
}

fn open_workspace(db_path: &str) -> Result<Repository> {
    let path = PathBuf::from(db_path);
    if !path.is_file() {
        anyhow::bail!(
            "workspace database {} does not exist; pass an existing neonexus.db",
            path.display()
        );
    }
    Repository::open(&path)
        .with_context(|| format!("failed to open workspace database {}", path.display()))
}

fn find_node_by_name(repository: &Repository, name: &str) -> Result<NodeConfig> {
    let nodes = repository
        .list_nodes()
        .context("failed to read nodes from workspace")?;
    nodes
        .into_iter()
        .find(|node| node.name == name)
        .with_context(|| format!("no node named '{name}' found in workspace"))
}
