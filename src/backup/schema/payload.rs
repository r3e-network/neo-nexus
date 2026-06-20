use serde::{Deserialize, Serialize};

use super::{
    EventBackup, FastSyncSnapshotBackup, NeoWalletProfileBackup, NodeBackup,
    RemoteServerProfileBackup, RuntimeCatalogProfileBackup, RuntimeSignerProfileBackup,
    WorkspaceSettingBackup,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceBackup {
    pub schema_version: u16,
    pub application: String,
    pub application_version: String,
    pub exported_at_unix: u64,
    #[serde(default)]
    pub workspace_settings: Vec<WorkspaceSettingBackup>,
    #[serde(default)]
    pub remote_servers: Vec<RemoteServerProfileBackup>,
    #[serde(default)]
    pub runtime_catalog_profiles: Vec<RuntimeCatalogProfileBackup>,
    #[serde(default)]
    pub runtime_signer_profiles: Vec<RuntimeSignerProfileBackup>,
    #[serde(default)]
    pub neo_wallet_profiles: Vec<NeoWalletProfileBackup>,
    #[serde(default)]
    pub fast_sync_snapshots: Vec<FastSyncSnapshotBackup>,
    pub nodes: Vec<NodeBackup>,
    pub events: Vec<EventBackup>,
}
