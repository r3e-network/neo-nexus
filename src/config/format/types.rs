use serde::Serialize;

use crate::types::NodeType;

use super::config_format;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeConfigProfile {
    pub network_magic: u32,
    pub seed_nodes: Vec<String>,
    pub validators_count: u8,
    pub committee_public_keys: Vec<String>,
    pub consensus_enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigFormat {
    Json,
    Yaml,
    Toml,
}

impl ConfigFormat {
    pub fn label(self) -> &'static str {
        match self {
            Self::Json => "JSON",
            Self::Yaml => "YAML",
            Self::Toml => "TOML",
        }
    }

    pub fn for_node_type(node_type: NodeType) -> Self {
        config_format(node_type)
    }

    pub(super) fn extension(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Yaml => "yml",
            Self::Toml => "toml",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedConfig {
    pub format: ConfigFormat,
    pub text: String,
}
