use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(in crate::private_network) struct DeploymentNodeManifest {
    pub(in crate::private_network) start_order: usize,
    pub(in crate::private_network) node_id: String,
    pub(in crate::private_network) name: String,
    pub(in crate::private_network) role: String,
    pub(in crate::private_network) runtime: String,
    pub(in crate::private_network) storage: String,
    pub(in crate::private_network) rpc_port: u16,
    pub(in crate::private_network) p2p_port: u16,
    pub(in crate::private_network) ws_port: Option<u16>,
    pub(in crate::private_network) binary_path: String,
    pub(in crate::private_network) arguments: Vec<String>,
    pub(in crate::private_network) working_dir: String,
    pub(in crate::private_network) config_path: String,
    #[serde(default)]
    pub(in crate::private_network) config_sha256: String,
    pub(in crate::private_network) command: String,
}
