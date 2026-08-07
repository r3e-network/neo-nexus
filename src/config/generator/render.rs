use anyhow::{Context, Result};
use serde_json::Value;

use crate::{
    catalog::PluginState,
    types::{NodeConfig, NodeType},
};

use super::super::format::{ConfigFormat, GenerationContext, RenderedConfig, RuntimeConfigProfile};
use super::{neo_cli::PluginSidecar, ConfigGenerator};

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

    /// The plugin configuration files a node needs beside its primary config.
    ///
    /// Only neo-cli has any: neo-go and neo-rs configure every service inside
    /// their single file. For neo-cli this is where the RPC port, the oracle
    /// service, the state service and dBFT consensus are actually set, so an
    /// export that writes only the primary file configures none of them.
    pub fn sidecars_for_node(node: &NodeConfig, plugins: &[PluginState]) -> Vec<PluginSidecar> {
        if node.node_type != NodeType::NeoCli {
            return Vec::new();
        }
        let enabled: Vec<_> = plugins
            .iter()
            .filter(|plugin| plugin.enabled)
            .map(|plugin| plugin.plugin_id)
            .collect();
        Self::neo_cli_sidecars(node, &enabled)
    }

    pub fn render_for_node(node: &NodeConfig, plugins: &[PluginState]) -> Result<RenderedConfig> {
        Self::render_for_node_with_profile(node, plugins, None)
    }

    pub fn render_for_node_with_profile(
        node: &NodeConfig,
        plugins: &[PluginState],
        profile: Option<&RuntimeConfigProfile>,
    ) -> Result<RenderedConfig> {
        Self::render_for_node_with_context(node, plugins, profile, &GenerationContext::default())
    }

    /// Renders a node's configuration for the duty it is being operated for.
    ///
    /// The duty is what switches on a service section, so a render without a
    /// context produces a plain relaying node — which is exactly what selecting
    /// a role used to produce, because the role never reached this function.
    pub fn render_for_node_with_context(
        node: &NodeConfig,
        plugins: &[PluginState],
        profile: Option<&RuntimeConfigProfile>,
        context: &GenerationContext,
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
                text: Self::neo_go_yaml_with_context(node, profile, context)?,
            }),
            NodeType::NeoRs => Ok(RenderedConfig {
                format: ConfigFormat::Toml,
                text: Self::neo_rs_toml_with_profile(node, profile)?,
            }),
        }
    }
}
