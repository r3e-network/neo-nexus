use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::{
    catalog::PluginState,
    types::{NodeConfig, NodeType},
};

use super::model::ConfigExport;
use crate::config::{
    format::{config_filename, RuntimeConfigProfile},
    generator::ConfigGenerator,
    validation::ConfigValidator,
};

pub struct ConfigExporter;

impl ConfigExporter {
    pub fn target_path(base_dir: impl AsRef<Path>, node: &NodeConfig) -> PathBuf {
        base_dir.as_ref().join(config_filename(node))
    }

    pub fn managed_target_path(node_work_dir: impl AsRef<Path>, node: &NodeConfig) -> PathBuf {
        let node_work_dir = node_work_dir.as_ref();
        match node.node_type {
            NodeType::NeoCli => node_work_dir.join("config.json"),
            NodeType::NeoGo | NodeType::NeoRs => {
                node_work_dir.join("config").join(config_filename(node))
            }
        }
    }

    pub fn write_node_config(
        base_dir: impl AsRef<Path>,
        node: &NodeConfig,
        plugins: &[PluginState],
    ) -> Result<ConfigExport> {
        let base_dir = base_dir.as_ref();
        Self::write_node_config_to_path(Self::target_path(base_dir, node), node, plugins)
    }

    pub fn write_node_config_to_path(
        path: impl AsRef<Path>,
        node: &NodeConfig,
        plugins: &[PluginState],
    ) -> Result<ConfigExport> {
        Self::write_node_config_to_path_with_profile(path, node, plugins, None)
    }

    pub fn write_node_config_to_path_with_profile(
        path: impl AsRef<Path>,
        node: &NodeConfig,
        plugins: &[PluginState],
        profile: Option<&RuntimeConfigProfile>,
    ) -> Result<ConfigExport> {
        let rendered = ConfigGenerator::render_for_node_with_profile(node, plugins, profile)?;
        let validation = ConfigValidator::validate_rendered_with_profile(node, &rendered, profile);
        if !validation.is_success() {
            anyhow::bail!(
                "generated {} config failed validation: {}",
                node.node_type,
                validation.operator_summary()
            );
        }

        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create config directory {}", parent.display())
            })?;
        }
        fs::write(&path, rendered.text.as_bytes())
            .with_context(|| format!("failed to write config {}", path.display()))?;

        Ok(ConfigExport {
            path,
            bytes_written: rendered.text.len(),
        })
    }
}
