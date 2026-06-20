use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeBackup {
    pub id: String,
    pub name: String,
    pub node_type: String,
    pub network: String,
    pub binary_path: String,
    pub args: Vec<String>,
    pub runtime_version: String,
    pub storage_engine: String,
    pub rpc_port: u16,
    pub p2p_port: u16,
    pub ws_port: Option<u16>,
    pub status: String,
    pub pid: Option<u32>,
    pub plugins: Vec<PluginBackup>,
    #[serde(default)]
    pub plugin_installations: Vec<PluginInstallationBackup>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginBackup {
    pub plugin_id: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginInstallationBackup {
    pub plugin_id: String,
    pub installed_path: String,
    pub manifest_path: String,
    pub source_path: String,
    pub sha256: String,
    pub package_bytes: u64,
    pub installed_files: usize,
    pub expanded_bytes: u64,
    pub installed_at_unix: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventBackup {
    pub id: i64,
    pub occurred_at_unix: u64,
    pub node_id: Option<String>,
    pub node_name: Option<String>,
    pub kind: String,
    pub severity: String,
    pub message: String,
}
