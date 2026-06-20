use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::cli::actions::config) struct NodeConfigCliSpec {
    pub(in crate::cli::actions::config) node_type: NodeType,
    pub(in crate::cli::actions::config) network: Network,
    pub(in crate::cli::actions::config) storage_engine: StorageEngine,
    pub(in crate::cli::actions::config) rpc_port: u16,
    pub(in crate::cli::actions::config) p2p_port: u16,
}

impl NodeConfigCliSpec {
    pub(in crate::cli::actions::config) fn from_args(args: &[String]) -> Result<Self> {
        let node_type = NodeType::from_str(&args[2])?;
        let network = Network::from_str(&args[3])?;
        let storage_engine = StorageEngine::from_str(&args[4])?;
        if !node_type.supports_storage_engine(storage_engine) {
            anyhow::bail!("{node_type} does not support {storage_engine} storage in NeoNexus");
        }
        let rpc_port = parse_u16_arg(&args[5], "rpc-port")?;
        let p2p_port = parse_u16_arg(&args[6], "p2p-port")?;
        validate_node_ports(rpc_port, p2p_port, None)?;

        Ok(Self {
            node_type,
            network,
            storage_engine,
            rpc_port,
            p2p_port,
        })
    }

    pub(in crate::cli::actions::config) fn generated_node(self) -> NodeConfig {
        NodeConfig {
            id: "generated-node-config".to_string(),
            name: format!("{} generated config", self.node_type),
            node_type: self.node_type,
            network: self.network,
            binary_path: default_runtime_binary_path(self.node_type),
            args: Vec::new(),
            runtime_version: "generated".to_string(),
            storage_engine: self.storage_engine,
            rpc_port: self.rpc_port,
            p2p_port: self.p2p_port,
            ws_port: None,
            status: NodeStatus::Stopped,
            pid: None,
        }
    }

    pub(in crate::cli::actions::config) fn validation_node(self) -> NodeConfig {
        NodeConfig {
            id: "config-validation".to_string(),
            name: "config validation".to_string(),
            node_type: self.node_type,
            network: self.network,
            binary_path: PathBuf::new(),
            args: Vec::new(),
            runtime_version: "external".to_string(),
            storage_engine: self.storage_engine,
            rpc_port: self.rpc_port,
            p2p_port: self.p2p_port,
            ws_port: None,
            status: NodeStatus::Stopped,
            pid: None,
        }
    }
}

fn default_runtime_binary_path(node_type: NodeType) -> PathBuf {
    match node_type {
        NodeType::NeoCli => PathBuf::from("neo-cli"),
        NodeType::NeoGo => PathBuf::from("neo-go"),
        NodeType::NeoRs => PathBuf::from("neo-node"),
    }
}

fn parse_u16_arg(value: &str, label: &str) -> Result<u16> {
    value
        .parse::<u16>()
        .with_context(|| format!("{label} must be a TCP port between 1 and 65535"))
}
