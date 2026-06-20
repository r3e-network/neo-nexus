use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::{
    catalog::PluginState,
    config::format::{config_filename, config_format},
    types::NodeConfig,
};

use super::super::{model::NodeConfigExportReport, node::ConfigExporter};

pub(super) fn export_node_configs(
    output_dir: &Path,
    nodes: &[(NodeConfig, Vec<PluginState>)],
) -> Result<Vec<NodeConfigExportReport>> {
    let mut reports = Vec::with_capacity(nodes.len());
    for (node, plugins) in nodes {
        reports.push(export_node_config(output_dir, node, plugins)?);
    }
    Ok(reports)
}

fn export_node_config(
    output_dir: &Path,
    node: &NodeConfig,
    plugins: &[PluginState],
) -> Result<NodeConfigExportReport> {
    let path = node_config_path(output_dir, node);
    let export = ConfigExporter::write_node_config_to_path(&path, node, plugins)
        .with_context(|| format!("failed to export config for {}", node.name))?;

    Ok(NodeConfigExportReport {
        node_id: node.id.clone(),
        node_name: node.name.clone(),
        node_type: node.node_type.to_string(),
        network: node.network.to_string(),
        storage_engine: node.storage_engine.to_string(),
        runtime_version: node.runtime_version.clone(),
        rpc_port: node.rpc_port,
        p2p_port: node.p2p_port,
        ws_port: node.ws_port,
        config_format: config_format(node.node_type).label().to_ascii_lowercase(),
        path: export.path.display().to_string(),
        bytes_written: export.bytes_written,
        plugin_count: plugins.len(),
        enabled_plugin_count: plugins.iter().filter(|plugin| plugin.enabled).count(),
    })
}

fn node_config_path(output_dir: &Path, node: &NodeConfig) -> PathBuf {
    output_dir
        .join("nodes")
        .join(&node.id)
        .join(config_filename(node))
}
