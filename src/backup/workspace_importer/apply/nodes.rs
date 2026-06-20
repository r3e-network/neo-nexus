use anyhow::Result;

use crate::{
    backup::{restore::restored_node, schema::WorkspaceBackup},
    repository::{Repository, RestoreNodeOutcome},
};

use super::counts::NodeImportCounts;

pub(super) fn restore_nodes(
    repository: &Repository,
    backup: &WorkspaceBackup,
) -> Result<NodeImportCounts> {
    let mut counts = NodeImportCounts::empty();

    for node_backup in &backup.nodes {
        let (node, plugins, plugin_installations) = restored_node(node_backup)?;
        counts.plugin_state_count += plugins.len();
        counts.plugin_installation_count += plugin_installations.len();
        match repository.restore_node_with_plugins(&node, &plugins)? {
            RestoreNodeOutcome::Created => counts.created_nodes += 1,
            RestoreNodeOutcome::Updated => counts.updated_nodes += 1,
        }
        repository.replace_plugin_installations(&node.id, &plugin_installations)?;
    }

    Ok(counts)
}
