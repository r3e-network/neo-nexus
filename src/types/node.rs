use std::path::PathBuf;

use super::{Network, NodeStatus, NodeType, StorageEngine};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewNode {
    pub name: String,
    pub node_type: NodeType,
    pub network: Network,
    pub binary_path: PathBuf,
    pub args: Vec<String>,
    pub runtime_version: String,
    pub storage_engine: StorageEngine,
    pub rpc_port: u16,
    pub p2p_port: u16,
    pub ws_port: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeConfig {
    pub id: String,
    pub name: String,
    pub node_type: NodeType,
    pub network: Network,
    pub binary_path: PathBuf,
    pub args: Vec<String>,
    pub runtime_version: String,
    pub storage_engine: StorageEngine,
    pub rpc_port: u16,
    pub p2p_port: u16,
    pub ws_port: Option<u16>,
    pub status: NodeStatus,
    pub pid: Option<u32>,
}
