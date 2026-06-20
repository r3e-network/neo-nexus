mod build;
mod model;

use anyhow::{Context, Result};

use crate::types::NodeConfig;

use super::{super::format::RuntimeConfigProfile, ConfigGenerator};

impl ConfigGenerator {
    pub fn neo_go_yaml(node: &NodeConfig) -> Result<String> {
        Self::neo_go_yaml_with_profile(node, None)
    }

    pub fn neo_go_yaml_with_profile(
        node: &NodeConfig,
        profile: Option<&RuntimeConfigProfile>,
    ) -> Result<String> {
        let config = build::neo_go_config(node, profile)?;
        serde_yaml::to_string(&config).context("failed to render neo-go YAML")
    }
}
