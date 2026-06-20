use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::snapshots::normalize_sha256;

use super::model::PluginPackageManifest;

pub fn validate_plugin_package_manifest(manifest: &PluginPackageManifest) -> Result<()> {
    if manifest.label.trim().is_empty() {
        anyhow::bail!("plugin package label is required");
    }
    if manifest.source_path.as_os_str().is_empty() {
        anyhow::bail!("plugin package source path is required");
    }
    let source_name = manifest
        .source_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !source_name.ends_with(".zip") {
        anyhow::bail!("plugin package source must be a .zip file");
    }
    normalize_sha256(&manifest.expected_sha256)?;
    Ok(())
}

pub(super) fn verified_plugin_source(path: &Path) -> Result<PathBuf> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("plugin package source {} was not found", path.display()))?;
    if !canonical.is_file() {
        anyhow::bail!(
            "plugin package source {} is not a file",
            canonical.display()
        );
    }
    Ok(canonical)
}
