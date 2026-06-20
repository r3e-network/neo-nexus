use std::{fs, path::Path};

use anyhow::{Context, Result};

use crate::types::NodeConfig;

use super::super::{
    archive::ZipInstallResult,
    model::{InstalledPluginManifest, PluginPackageManifest},
    PLUGIN_CONTROL_DIR,
};

pub(super) struct InstallManifestRequest<'a> {
    pub(super) manifest: &'a PluginPackageManifest,
    pub(super) node: &'a NodeConfig,
    pub(super) source_path: &'a Path,
    pub(super) sha256: &'a str,
    pub(super) package_bytes: u64,
    pub(super) install_result: &'a ZipInstallResult,
    pub(super) installed_at_unix: u64,
    pub(super) staging_dir: &'a Path,
}

pub(super) fn write_install_manifest(request: InstallManifestRequest<'_>) -> Result<()> {
    let staging_manifest_path = request
        .staging_dir
        .join(PLUGIN_CONTROL_DIR)
        .join("manifest.json");
    if let Some(parent) = staging_manifest_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create plugin manifest directory {}",
                parent.display()
            )
        })?;
    }
    let manifest_text = serde_json::to_string_pretty(&InstalledPluginManifest {
        schema_version: 1,
        plugin_id: request.manifest.plugin_id.to_string(),
        label: request.manifest.label.trim().to_string(),
        node_id: request.node.id.clone(),
        node_name: request.node.name.clone(),
        node_type: request.node.node_type.to_string(),
        source_path: request.source_path.display().to_string(),
        sha256: request.sha256.to_string(),
        package_bytes: request.package_bytes,
        installed_files: request.install_result.installed_files,
        expanded_bytes: request.install_result.expanded_bytes,
        installed_at_unix: request.installed_at_unix,
    })
    .context("failed to render plugin installation manifest")?;
    fs::write(&staging_manifest_path, manifest_text.as_bytes()).with_context(|| {
        format!(
            "failed to write plugin installation manifest {}",
            staging_manifest_path.display()
        )
    })
}
