use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;

use super::{counts::BackupValidationCounts, uniqueness::insert_unique_value};
use crate::{
    backup::{
        restore::restored_node,
        schema::{NodeBackup, WorkspaceBackup},
    },
    catalog::PluginState,
    plugins::PluginInstallation,
    types::NodeConfig,
};

pub(super) fn validate_backup_nodes(backup: &WorkspaceBackup) -> Result<BackupValidationCounts> {
    let mut node_ids = BTreeSet::new();
    let mut ports = BTreeMap::new();
    let mut counts = BackupValidationCounts::default();

    for node_backup in &backup.nodes {
        validate_backup_node(node_backup, &mut node_ids, &mut ports, &mut counts)?;
    }

    Ok(counts)
}

fn validate_backup_node(
    node_backup: &NodeBackup,
    node_ids: &mut BTreeSet<String>,
    ports: &mut BTreeMap<u16, (String, &'static str)>,
    counts: &mut BackupValidationCounts,
) -> Result<()> {
    insert_unique_value(node_ids, "node id", node_backup.id.trim())?;
    let (node, plugins, plugin_installations) = restored_node(node_backup)?;
    validate_node_plugin_inventory(&node, &plugins, &plugin_installations)?;
    validate_node_ports_are_unique(&node, ports)?;
    counts.plugin_state_count += plugins.len();
    counts.plugin_installation_count += plugin_installations.len();
    Ok(())
}

fn validate_node_plugin_inventory(
    node: &NodeConfig,
    plugins: &[PluginState],
    plugin_installations: &[PluginInstallation],
) -> Result<()> {
    let mut plugin_ids = BTreeSet::new();
    for plugin in plugins {
        let plugin_id = plugin.plugin_id.to_string();
        if !plugin_ids.insert(plugin_id.clone()) {
            anyhow::bail!(
                "backup node {} has duplicate plugin state {}",
                node.name,
                plugin_id
            );
        }
    }

    let mut installation_ids = BTreeSet::new();
    for installation in plugin_installations {
        if installation.node_id != node.id {
            anyhow::bail!(
                "backup node {} has plugin installation for another node {}",
                node.name,
                installation.node_id
            );
        }
        let plugin_id = installation.plugin_id.to_string();
        if !installation_ids.insert(plugin_id.clone()) {
            anyhow::bail!(
                "backup node {} has duplicate plugin installation {}",
                node.name,
                plugin_id
            );
        }
    }

    Ok(())
}

fn validate_node_ports_are_unique(
    node: &NodeConfig,
    ports: &mut BTreeMap<u16, (String, &'static str)>,
) -> Result<()> {
    for (label, port) in backup_node_ports(node) {
        if let Some((existing_node, existing_label)) =
            ports.insert(port, (node.name.clone(), label))
        {
            anyhow::bail!(
                "duplicate backup node port {port}: {existing_node} {existing_label} conflicts with {} {label}",
                node.name
            );
        }
    }
    Ok(())
}

fn backup_node_ports(node: &NodeConfig) -> Vec<(&'static str, u16)> {
    let mut ports = vec![("RPC", node.rpc_port), ("P2P", node.p2p_port)];
    if let Some(port) = node.ws_port {
        ports.push(("WS", port));
    }
    ports
}
