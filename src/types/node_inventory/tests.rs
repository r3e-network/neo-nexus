use std::path::PathBuf;

use super::*;
use crate::types::{Network, NodeType, StorageEngine};

#[test]
fn node_inventory_filter_matches_operational_fields() {
    let nodes = [node("rpc-1", "RPC Alpha", NodeStatus::Running, 10332)];

    assert_ids(&nodes, NodeInventoryFilter::new(None, "alpha"), &["rpc-1"]);
    assert_ids(&nodes, NodeInventoryFilter::new(None, "neo-rs"), &["rpc-1"]);
    assert_ids(
        &nodes,
        NodeInventoryFilter::new(None, "testnet"),
        &["rpc-1"],
    );
    assert_ids(
        &nodes,
        NodeInventoryFilter::new(None, "running"),
        &["rpc-1"],
    );
    assert_ids(&nodes, NodeInventoryFilter::new(None, "10332"), &["rpc-1"]);
    assert_ids(&nodes, NodeInventoryFilter::new(None, "3.8.1"), &["rpc-1"]);
}

#[test]
fn node_inventory_filter_combines_status_and_query() {
    let nodes = [
        node("rpc-1", "RPC Alpha", NodeStatus::Running, 10332),
        node("rpc-2", "RPC Beta", NodeStatus::Stopped, 20332),
        node("seed-1", "Seed Beta", NodeStatus::Running, 30332),
    ];
    let filter = NodeInventoryFilter::new(Some(NodeStatus::Running), "beta");

    assert_ids(&nodes, filter, &["seed-1"]);
}

fn assert_ids(nodes: &[NodeConfig], filter: NodeInventoryFilter, ids: &[&str]) {
    let filtered = filter_nodes(nodes, &filter);
    let actual = filtered
        .iter()
        .map(|node| node.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(actual.as_slice(), ids);
}

fn node(id: &str, name: &str, status: NodeStatus, rpc_port: u16) -> NodeConfig {
    NodeConfig {
        id: id.to_string(),
        name: name.to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: PathBuf::from("/opt/neo-node"),
        args: vec!["--config".to_string(), "config.toml".to_string()],
        runtime_version: "3.8.1".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port,
        p2p_port: rpc_port + 1,
        ws_port: Some(rpc_port + 2),
        status,
        pid: None,
    }
}
