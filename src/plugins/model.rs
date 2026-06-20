use std::path::PathBuf;

use serde::Serialize;

use crate::catalog::PluginId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginPackageManifest {
    pub plugin_id: PluginId,
    pub label: String,
    pub source_path: PathBuf,
    pub expected_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginInstallation {
    pub node_id: String,
    pub plugin_id: PluginId,
    pub installed_path: PathBuf,
    pub manifest_path: PathBuf,
    pub source_path: PathBuf,
    pub sha256: String,
    pub package_bytes: u64,
    pub installed_files: usize,
    pub expanded_bytes: u64,
    pub installed_at_unix: u64,
}

#[derive(Serialize)]
pub(super) struct InstalledPluginManifest {
    pub(super) schema_version: u32,
    pub(super) plugin_id: String,
    pub(super) label: String,
    pub(super) node_id: String,
    pub(super) node_name: String,
    pub(super) node_type: String,
    pub(super) source_path: String,
    pub(super) sha256: String,
    pub(super) package_bytes: u64,
    pub(super) installed_files: usize,
    pub(super) expanded_bytes: u64,
    pub(super) installed_at_unix: u64,
}
