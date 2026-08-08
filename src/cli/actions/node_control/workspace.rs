//! Locating the workspace and the nodes inside it.

use std::path::PathBuf;

use super::super::*;

pub(super) fn open_workspace(db_path: &str) -> Result<Repository> {
    Repository::open(PathBuf::from(db_path))
        .with_context(|| format!("failed to open workspace database {db_path}"))
}

pub(super) fn node_by_name(repository: &Repository, name: &str) -> Result<NodeConfig> {
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
pub(super) fn workspace_child_dir(repository: &Repository, child: &str) -> PathBuf {
    repository
        .db_path()
        .parent()
        .map_or_else(|| PathBuf::from(child), |parent| parent.join(child))
}
