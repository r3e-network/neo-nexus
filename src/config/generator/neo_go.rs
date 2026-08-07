mod build;
mod model;
mod services;

use anyhow::{Context, Result};

use crate::types::NodeConfig;

use super::{
    super::format::{GenerationContext, RuntimeConfigProfile},
    ConfigGenerator,
};

impl ConfigGenerator {
    pub fn neo_go_yaml(node: &NodeConfig) -> Result<String> {
        Self::neo_go_yaml_with_profile(node, None)
    }

    pub fn neo_go_yaml_with_profile(
        node: &NodeConfig,
        profile: Option<&RuntimeConfigProfile>,
    ) -> Result<String> {
        Self::neo_go_yaml_with_context(node, profile, &GenerationContext::default())
    }

    pub fn neo_go_yaml_with_context(
        node: &NodeConfig,
        profile: Option<&RuntimeConfigProfile>,
        context: &GenerationContext,
    ) -> Result<String> {
        let config = build::neo_go_config(node, profile, context)?;
        serde_yaml::to_string(&config).context("failed to render neo-go YAML")
    }
}
