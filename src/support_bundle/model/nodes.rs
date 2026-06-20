use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SupportBundleNode {
    pub id: String,
    pub name: String,
    pub node_type: String,
    pub network: String,
    pub runtime_version: String,
    pub storage_engine: String,
    pub status: String,
    pub pid: Option<u32>,
    pub binary_path: String,
    pub argument_count: usize,
    pub redacted_args: Vec<String>,
    pub external_config_arg: bool,
    pub rpc_port: u16,
    pub p2p_port: u16,
    pub ws_port: Option<u16>,
}
