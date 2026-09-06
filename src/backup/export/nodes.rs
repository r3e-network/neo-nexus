use anyhow::Result;

use crate::{
    catalog::PluginState,
    plugins::PluginInstallation,
    repository::Repository,
    types::{NodeConfig, NodeStatus},
};

use super::super::schema::{
    NodeBackup, NodeSignerKeyBackup, PluginBackup, PluginInstallationBackup,
};

pub(in crate::backup) fn node_backup(
    repository: &Repository,
    node: NodeConfig,
) -> Result<NodeBackup> {
    let retained_runtime = repository.quarantined_runtime_spec(&node.id)?;
    let plugins = repository
        .list_plugin_states(&node.id)?
        .into_iter()
        .map(plugin_backup)
        .collect();
    let plugin_installations = repository
        .list_plugin_installations(&node.id)?
        .into_iter()
        .map(plugin_installation_backup)
        .collect();

    // Duty and wallet binding travel with the node: regenerating a config
    // without them turns a consensus node back into a relay.
    let role = repository
        .load_node_role(&node.id)?
        .map(|role| role.persist_key().to_string());
    let wallet_profile_id = repository.load_node_wallet(&node.id)?;
    let signer_key = repository
        .load_node_signer_key(&node.id)?
        .map(|key| NodeSignerKeyBackup {
            backend_id: key.backend_id,
            key_id: key.key_id,
        });

    let (binary_path, args) = retained_runtime
        .map(|runtime| (runtime.binary_path, runtime.args))
        .unwrap_or_else(|| (node.binary_path.clone(), node.args.clone()));

    Ok(NodeBackup {
        id: node.id,
        name: node.name,
        node_type: node.node_type.to_string(),
        network: node.network.to_string(),
        binary_path: binary_path.display().to_string(),
        args,
        runtime_version: node.runtime_version,
        storage_engine: node.storage_engine.to_string(),
        rpc_port: node.rpc_port,
        p2p_port: node.p2p_port,
        ws_port: node.ws_port,
        status: backup_status(node.status),
        pid: node.pid,
        plugins,
        plugin_installations,
        role,
        wallet_profile_id,
        signer_key,
    })
}

fn plugin_backup(plugin: PluginState) -> PluginBackup {
    PluginBackup {
        plugin_id: plugin.plugin_id.to_string(),
        enabled: plugin.enabled,
    }
}

fn plugin_installation_backup(installation: PluginInstallation) -> PluginInstallationBackup {
    PluginInstallationBackup {
        plugin_id: installation.plugin_id.to_string(),
        installed_path: installation.installed_path.display().to_string(),
        manifest_path: installation.manifest_path.display().to_string(),
        source_path: installation.source_path.display().to_string(),
        sha256: installation.sha256,
        package_bytes: installation.package_bytes,
        installed_files: installation.installed_files,
        expanded_bytes: installation.expanded_bytes,
        installed_at_unix: installation.installed_at_unix,
    }
}

fn backup_status(status: NodeStatus) -> String {
    status.to_string()
}
