use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkspaceBackupExport {
    pub path: PathBuf,
    pub schema_version: u16,
    pub application_version: String,
    pub exported_at_unix: u64,
    pub bytes_written: usize,
    pub node_count: usize,
    pub plugin_state_count: usize,
    pub plugin_installation_count: usize,
    pub workspace_setting_count: usize,
    pub remote_server_count: usize,
    pub runtime_catalog_profile_count: usize,
    pub runtime_signer_profile_count: usize,
    pub neo_wallet_profile_count: usize,
    pub fast_sync_snapshot_count: usize,
    pub event_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkspaceBackupImport {
    pub source_path: Option<PathBuf>,
    pub created_nodes: usize,
    pub updated_nodes: usize,
    pub plugin_state_count: usize,
    pub plugin_installation_count: usize,
    pub workspace_setting_count: usize,
    pub remote_server_count: usize,
    pub runtime_catalog_profile_count: usize,
    pub runtime_signer_profile_count: usize,
    pub neo_wallet_profile_count: usize,
    pub fast_sync_snapshot_count: usize,
    pub event_count: usize,
    pub schema_version: u16,
    pub exported_at_unix: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkspaceBackupValidation {
    pub source_path: Option<PathBuf>,
    pub schema_version: u16,
    pub application_version: String,
    pub exported_at_unix: u64,
    pub node_count: usize,
    pub plugin_state_count: usize,
    pub plugin_installation_count: usize,
    pub workspace_setting_count: usize,
    pub remote_server_count: usize,
    pub runtime_catalog_profile_count: usize,
    pub runtime_signer_profile_count: usize,
    pub neo_wallet_profile_count: usize,
    pub fast_sync_snapshot_count: usize,
    pub event_count: usize,
}
