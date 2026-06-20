use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceSettingBackup {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteServerProfileBackup {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub description: String,
    pub enabled: bool,
    pub created_at_unix: u64,
    pub updated_at_unix: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeCatalogProfileBackup {
    pub id: String,
    pub label: String,
    pub source: String,
    pub signature_source: Option<String>,
    pub ed25519_public_key: Option<String>,
    pub max_bytes: u64,
    pub enabled: bool,
    pub last_loaded_at_unix: Option<u64>,
    pub last_signature_verified: Option<bool>,
    pub last_bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeSignerProfileBackup {
    pub id: String,
    pub label: String,
    pub ed25519_public_key: String,
    pub enabled: bool,
    pub created_at_unix: u64,
    pub last_used_at_unix: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeoWalletProfileBackup {
    pub id: String,
    pub label: String,
    pub source_path: String,
    pub wallet_version: Option<String>,
    pub primary_address: String,
    pub contract_public_keys: Vec<String>,
    pub wallet_sha256: String,
    pub account_count: usize,
    pub encrypted_account_count: usize,
    pub default_account_count: usize,
    pub watch_only_account_count: usize,
    pub validated_at_unix: u64,
    pub last_used_at_unix: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FastSyncSnapshotBackup {
    pub id: String,
    pub label: String,
    pub network: String,
    pub node_type: String,
    pub source_path: String,
    pub source_url: Option<String>,
    pub download_file_name: Option<String>,
    pub download_max_bytes: u64,
    pub expected_sha256: String,
}
