//! Neo-rs Cargo feature definitions
//!
//! Neo-rs uses Cargo features instead of runtime plugins.
//! Features require recompilation when toggled.

use serde::{Deserialize, Serialize};

/// Neo-rs Cargo feature definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeoRsFeatureDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default = "default_requires_rebuild")]
    pub requires_rebuild: bool,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}
fn default_requires_rebuild() -> bool {
    true
}

impl NeoRsFeatureDefinition {
    pub fn new(id: &str, name: &str, description: &str, deps: Vec<&str>) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            requires_rebuild: !deps.is_empty(),
            dependencies: deps.iter().map(|s| s.to_string()).collect(),
            enabled: true,
        }
    }
}

pub fn get_available_features() -> Vec<NeoRsFeatureDefinition> {
    vec![
        NeoRsFeatureDefinition::new(
            "fast-sync",
            "Fast Sync",
            "Optimized sync",
            vec!["tokio-parallel"],
        ),
        NeoRsFeatureDefinition::new(
            "rocksdb-backend",
            "RocksDB Backend",
            "Uses RocksDB storage",
            vec!["rocksdb", "sled"],
        ),
        NeoRsFeatureDefinition::new("rpc-full-api", "Full RPC", "Complete RPC endpoints", vec![]),
    ]
}

pub fn toggle_feature(
    feature_id: &str,
    _enabled: bool,
    _ctx: &crate::config::GenerationContext,
) -> anyhow::Result<()> {
    anyhow::bail!(
        "NeoRs feature '{feature_id}' was not changed: automated Cargo feature management is not implemented and the runtime source workspace is not provided. No files were changed. Select features supported by your NeoRs version in its Cargo build configuration, rebuild the runtime, deploy the binary and restart."
    )
}

pub fn generate_build_instructions(_ctx: &crate::config::GenerationContext) -> String {
    "No NeoRs features have been changed. Select supported features in the runtime's source workspace, rebuild with Cargo, deploy the binary and restart.".to_string()
}

pub fn supports_features(node_type: crate::types::NodeType) -> bool {
    matches!(node_type, crate::types::NodeType::NeoRs)
}
