use crate::app::domain::{NewNode, NodeConfig, RuntimeInstallation};

pub(super) fn runtime_installation_node_input(
    node: &NodeConfig,
    installation: &RuntimeInstallation,
) -> NewNode {
    NewNode {
        name: node.name.clone(),
        node_type: node.node_type,
        network: node.network,
        binary_path: installation.binary_path.clone(),
        args: node.args.clone(),
        runtime_version: installation.version.clone(),
        storage_engine: node.storage_engine,
        rpc_port: node.rpc_port,
        p2p_port: node.p2p_port,
        ws_port: node.ws_port,
    }
}
