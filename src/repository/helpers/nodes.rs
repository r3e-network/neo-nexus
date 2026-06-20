use anyhow::Result;
use uuid::Uuid;

use crate::types::{validate_node_ports, NewNode, NodeConfig, NodeStatus};

pub(in crate::repository) fn new_node_config(input: NewNode) -> Result<NodeConfig> {
    validate_node_input(&input)?;
    Ok(NodeConfig {
        id: format!("node-{}", Uuid::new_v4()),
        name: input.name.trim().to_string(),
        node_type: input.node_type,
        network: input.network,
        binary_path: input.binary_path,
        args: input.args,
        runtime_version: normalize_runtime_version(&input.runtime_version),
        storage_engine: input.storage_engine,
        rpc_port: input.rpc_port,
        p2p_port: input.p2p_port,
        ws_port: input.ws_port,
        status: NodeStatus::Stopped,
        pid: None,
    })
}

pub(in crate::repository) fn validate_node_input(input: &NewNode) -> Result<()> {
    if input.name.trim().is_empty() {
        anyhow::bail!("node name is required");
    }
    if input.binary_path.as_os_str().is_empty() {
        anyhow::bail!("node binary path is required");
    }
    validate_node_ports(input.rpc_port, input.p2p_port, input.ws_port)?;
    if !input
        .node_type
        .supports_storage_engine(input.storage_engine)
    {
        anyhow::bail!(
            "{} does not support {} storage in NeoNexus",
            input.node_type,
            input.storage_engine
        );
    }
    Ok(())
}

pub(crate) fn validate_node_config(node: &NodeConfig) -> Result<()> {
    if node.id.trim().is_empty() {
        anyhow::bail!("node id is required");
    }
    let input = NewNode {
        name: node.name.clone(),
        node_type: node.node_type,
        network: node.network,
        binary_path: node.binary_path.clone(),
        args: node.args.clone(),
        runtime_version: node.runtime_version.clone(),
        storage_engine: node.storage_engine,
        rpc_port: node.rpc_port,
        p2p_port: node.p2p_port,
        ws_port: node.ws_port,
    };
    validate_node_input(&input)
}

pub(in crate::repository) fn normalize_runtime_version(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "latest".to_string()
    } else {
        trimmed.to_string()
    }
}
