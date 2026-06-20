use std::{collections::BTreeMap, path::PathBuf};

use crate::{catalog::PluginState, roles::PrivateNetworkPlan, types::NodeConfig};

use super::super::CommitteeRoster;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateNetworkDeploymentExport {
    pub root_path: PathBuf,
    pub manifest_path: PathBuf,
    pub start_order_path: PathBuf,
    pub runbook_path: PathBuf,
    pub wallet_provisioning_path: PathBuf,
    pub wallet_instructions_path: PathBuf,
    pub preflight_unix_path: PathBuf,
    pub preflight_windows_path: PathBuf,
    pub health_unix_path: PathBuf,
    pub health_windows_path: PathBuf,
    pub start_unix_path: PathBuf,
    pub stop_unix_path: PathBuf,
    pub start_windows_path: PathBuf,
    pub stop_windows_path: PathBuf,
    pub node_count: usize,
    pub config_count: usize,
    pub network_magic: u32,
    pub validators_count: u8,
    pub bytes_written: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateNetworkDeploymentRequest {
    pub plan: PrivateNetworkPlan,
    pub nodes: Vec<NodeConfig>,
    pub plugin_states: BTreeMap<String, Vec<PluginState>>,
    pub committee: Option<CommitteeRoster>,
    pub output_dir: PathBuf,
    pub node_root_dir: PathBuf,
}

pub struct PrivateNetworkDeploymentExporter;
