use std::{path::PathBuf, str::FromStr};

use anyhow::{Context, Result};

use crate::{
    catalog::{PluginId, PluginState},
    plugins::PluginInstallation,
    repository::validate_node_config,
    types::{Network, NodeConfig, NodeStatus, NodeType, StorageEngine},
};

use super::super::schema::{NodeBackup, PluginInstallationBackup};

pub(in crate::backup) fn restored_node(
    backup: &NodeBackup,
) -> Result<(NodeConfig, Vec<PluginState>, Vec<PluginInstallation>)> {
    if backup.id.trim().is_empty() {
        anyhow::bail!("backup node id is required");
    }

    let _backup_status = NodeStatus::from_str(&backup.status)
        .with_context(|| format!("backup node {} has invalid status", backup.id))?;
    let plugins = backup
        .plugins
        .iter()
        .map(restored_plugin_state)
        .collect::<Result<Vec<_>>>()?;
    let plugin_installations = backup
        .plugin_installations
        .iter()
        .map(|installation| restored_plugin_installation(&backup.id, installation))
        .collect::<Result<Vec<_>>>()?;

    let node = NodeConfig {
        id: backup.id.clone(),
        name: backup.name.clone(),
        node_type: NodeType::from_str(&backup.node_type)
            .with_context(|| format!("backup node {} has invalid runtime", backup.id))?,
        network: Network::from_str(&backup.network)
            .with_context(|| format!("backup node {} has invalid network", backup.id))?,
        binary_path: PathBuf::from(&backup.binary_path),
        args: backup.args.clone(),
        runtime_version: backup.runtime_version.clone(),
        storage_engine: StorageEngine::from_str(&backup.storage_engine)
            .with_context(|| format!("backup node {} has invalid storage", backup.id))?,
        rpc_port: backup.rpc_port,
        p2p_port: backup.p2p_port,
        ws_port: backup.ws_port,
        status: NodeStatus::Stopped,
        pid: None,
    };
    validate_node_config(&node)?;

    Ok((node, plugins, plugin_installations))
}

fn restored_plugin_state(backup: &super::super::schema::PluginBackup) -> Result<PluginState> {
    Ok(PluginState {
        plugin_id: PluginId::from_str(&backup.plugin_id)?,
        enabled: backup.enabled,
    })
}

fn restored_plugin_installation(
    node_id: &str,
    backup: &PluginInstallationBackup,
) -> Result<PluginInstallation> {
    let installation = PluginInstallation {
        node_id: node_id.to_string(),
        plugin_id: PluginId::from_str(&backup.plugin_id)?,
        installed_path: PathBuf::from(&backup.installed_path),
        manifest_path: PathBuf::from(&backup.manifest_path),
        source_path: PathBuf::from(&backup.source_path),
        sha256: backup.sha256.clone(),
        package_bytes: backup.package_bytes,
        installed_files: backup.installed_files,
        expanded_bytes: backup.expanded_bytes,
        installed_at_unix: backup.installed_at_unix,
    };
    validate_plugin_installation_backup(&installation)?;
    Ok(installation)
}

fn validate_plugin_installation_backup(installation: &PluginInstallation) -> Result<()> {
    if installation.node_id.trim().is_empty() {
        anyhow::bail!("backup plugin installation node id is required");
    }
    if installation.installed_path.as_os_str().is_empty() {
        anyhow::bail!("backup plugin installation installed path is required");
    }
    if installation.manifest_path.as_os_str().is_empty() {
        anyhow::bail!("backup plugin installation manifest path is required");
    }
    if installation.source_path.as_os_str().is_empty() {
        anyhow::bail!("backup plugin installation source path is required");
    }
    if !is_sha256_hex(&installation.sha256) {
        anyhow::bail!(
            "backup plugin installation {} has invalid SHA-256",
            installation.plugin_id
        );
    }
    if installation.package_bytes == 0 {
        anyhow::bail!(
            "backup plugin installation {} package bytes must be greater than 0",
            installation.plugin_id
        );
    }
    if installation.installed_files == 0 {
        anyhow::bail!(
            "backup plugin installation {} file count must be greater than 0",
            installation.plugin_id
        );
    }
    Ok(())
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
