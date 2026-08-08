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
        let (node, plugins, plugin_installations, bindings) = restored_node(node_backup)?;
        counts.plugin_state_count += plugins.len();
        counts.plugin_installation_count += plugin_installations.len();
        match repository.restore_node_with_plugins(&node, &plugins)? {
            RestoreNodeOutcome::Created => counts.created_nodes += 1,
            RestoreNodeOutcome::Updated => counts.updated_nodes += 1,
        }
        repository.replace_plugin_installations(&node.id, &plugin_installations)?;
        // Duty and wallet, restored explicitly. Regenerating a config without
        // them turns a consensus node back into a relay, and the old summary had
        // no counter that could come back zero to say so.
        repository.set_node_role(&node.id, bindings.role)?;
        if bindings.role.is_some() {
            counts.role_count += 1;
        }
        repository.set_node_wallet(&node.id, bindings.wallet_profile_id.as_deref())?;
        if bindings.wallet_profile_id.is_some() {
            counts.wallet_binding_count += 1;
        }
    }

    Ok(counts)
}
