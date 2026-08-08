use std::path::PathBuf;

use super::*;
use crate::types::{Network, NodeStatus, NodeType};

fn node(node_type: NodeType, network: Network) -> NodeConfig {
    NodeConfig {
        id: "node".to_string(),
        name: "node".to_string(),
        node_type,
        network,
        binary_path: PathBuf::from("/opt/node"),
        args: Vec::new(),
        runtime_version: "test".to_string(),
        storage_engine: node_type.default_storage_engine(),
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: None,
        status: NodeStatus::Stopped,
        pid: None,
    }
}

/// Neo N3 proves its chain in every peer handshake, so this check would only be
/// noise there. It exists because Neo X has no equivalent guard.
#[test]
fn neo_n3_nodes_get_no_chain_identity_check() {
    for node_type in [NodeType::NeoCli, NodeType::NeoGo, NodeType::NeoRs] {
        assert!(chain_identity_checks(&node(node_type, Network::Mainnet)).is_empty());
    }
}

#[test]
fn a_neox_node_is_told_its_chain_id_and_its_genesis_anchor() {
    let checks = chain_identity_checks(&node(NodeType::NeoXGeth, Network::Mainnet));
    assert_eq!(checks.len(), 2);

    let identity = &checks[0];
    assert_eq!(identity.title, "Chain identity");
    assert!(identity.detail.contains("47763"));
    assert!(identity.detail.contains("5s blocks"));
    assert!(identity.detail.contains("7 dBFT"));

    let genesis = &checks[1];
    assert_eq!(genesis.title, "Genesis anchor");
    assert_eq!(genesis.severity, CheckSeverity::Warning);
    assert!(genesis
        .detail
        .contains("0x2ee57478315c7d3182997a812d7885dafee48612cd88cb30b615847b0dd8dbd7"));
}

/// The testnet anchor must not be the mainnet one: a node verified against the
/// wrong hash is verified against nothing.
#[test]
fn each_public_network_carries_its_own_anchor() {
    let mainnet = chain_identity_checks(&node(NodeType::NeoXReth, Network::Mainnet));
    let testnet = chain_identity_checks(&node(NodeType::NeoXReth, Network::Testnet));
    assert_ne!(mainnet[0].detail, testnet[0].detail);
    assert_ne!(mainnet[1].detail, testnet[1].detail);
    assert!(testnet[0].detail.contains("12227332"));
}

/// NeoNexus does not generate a Neo X genesis, and the check has to say so
/// rather than imply one is coming.
#[test]
fn a_private_network_is_told_it_must_supply_its_own_genesis() {
    let checks = chain_identity_checks(&node(NodeType::NeoXGeth, Network::Private));
    let genesis = &checks[1];
    assert!(genesis.detail.contains("its own genesis file"));
    assert!(!genesis.detail.contains("0x"), "no anchor can be promised");
}

/// Both clients join the same chain, so both get the same identity.
#[test]
fn both_neox_clients_report_the_same_chain() {
    let geth = chain_identity_checks(&node(NodeType::NeoXGeth, Network::Mainnet));
    let reth = chain_identity_checks(&node(NodeType::NeoXReth, Network::Mainnet));
    assert_eq!(geth[0].detail, reth[0].detail);
    assert_eq!(geth[1].detail, reth[1].detail);
}
