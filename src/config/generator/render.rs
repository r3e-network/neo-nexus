use anyhow::{Context, Result};
use serde_json::Value;

use crate::{
    catalog::PluginState,
    types::{NodeConfig, NodeType},
};

use super::super::format::{ConfigFormat, RenderedConfig, RuntimeConfigProfile};
use super::ConfigGenerator;

impl ConfigGenerator {
    pub fn for_node(node: &NodeConfig, plugins: &[PluginState]) -> Result<Value> {
        Self::for_node_with_profile(node, plugins, None)
    }

    pub fn for_node_with_profile(
        node: &NodeConfig,
        plugins: &[PluginState],
        profile: Option<&RuntimeConfigProfile>,
    ) -> Result<Value> {
        match node.node_type {
            NodeType::NeoCli => Self::neo_cli_with_profile(node, plugins, profile),
            NodeType::NeoGo => anyhow::bail!("neo-go configuration is YAML, not JSON"),
            NodeType::NeoRs => anyhow::bail!("neo-rs configuration is TOML, not JSON"),
        }
    }

    pub fn render_for_node(node: &NodeConfig, plugins: &[PluginState]) -> Result<RenderedConfig> {
        Self::render_for_node_with_profile(node, plugins, None)
    }

    pub fn render_for_node_with_profile(
        node: &NodeConfig,
        plugins: &[PluginState],
        profile: Option<&RuntimeConfigProfile>,
    ) -> Result<RenderedConfig> {
        match node.node_type {
            NodeType::NeoCli => {
                let value = Self::for_node_with_profile(node, plugins, profile)?;
                Ok(RenderedConfig {
                    format: ConfigFormat::Json,
                    text: serde_json::to_string_pretty(&value)
                        .context("failed to render config JSON")?,
                })
            }
            NodeType::NeoGo => Ok(RenderedConfig {
                format: ConfigFormat::Yaml,
                text: Self::neo_go_yaml_with_profile(node, profile)?,
            }),
            NodeType::NeoRs => Ok(RenderedConfig {
                format: ConfigFormat::Toml,
                text: Self::neo_rs_toml_with_profile(node, profile)?,
            }),
        }
    }
}
