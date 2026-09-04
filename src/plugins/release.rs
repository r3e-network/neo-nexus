use crate::types::NodeConfig;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

/// Operator-supplied release identity is retained alongside the verified archive
/// digest. Exact compatibility avoids guessing that unrelated Neo releases load
/// the same plugin ABI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginReleaseMetadata {
    pub version: String,
    pub compatible_runtime_versions: Vec<String>,
}

impl PluginReleaseMetadata {
    pub fn validate_for(&self, node: &NodeConfig) -> Result<()> {
        if self.version.trim().is_empty() {
            anyhow::bail!("plugin version is required");
        }
        if unpinned(&self.version) || unpinned(&node.runtime_version) {
            anyhow::bail!("pin an explicit plugin version and node runtime version before declaring compatibility");
        }
        if !self
            .compatible_runtime_versions
            .iter()
            .any(|version| normalized(version) == normalized(&node.runtime_version))
        {
            anyhow::bail!(
                "plugin {} is not declared compatible with neo-cli {}; supported runtimes: {}",
                self.version,
                node.runtime_version,
                self.compatible_runtime_versions.join(", ")
            );
        }
        Ok(())
    }
}

fn normalized(version: &str) -> &str {
    version.trim().trim_start_matches('v')
}

fn unpinned(version: &str) -> bool {
    matches!(
        version.trim().to_ascii_lowercase().as_str(),
        "" | "latest" | "external" | "generated" | "unknown" | "*"
    )
}

pub fn installed_plugin_release(manifest_path: &Path) -> Result<Option<PluginReleaseMetadata>> {
    let value: serde_json::Value = serde_json::from_slice(
        &fs::read(manifest_path).context("plugin installation manifest is unavailable")?,
    )?;
    value
        .get("release")
        .filter(|value| !value.is_null())
        .map(|value| serde_json::from_value(value.clone()).map_err(Into::into))
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn leading_v_is_only_normalization() {
        assert_eq!(normalized(" v3.8.0 "), "3.8.0");
        assert_ne!(normalized("3.8.0-rc.1"), normalized("3.8.0"));
    }
}
