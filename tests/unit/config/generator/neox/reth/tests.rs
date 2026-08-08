use std::path::PathBuf;

use super::*;
use crate::types::{NodeStatus, NodeType, StorageEngine};

fn node(network: Network) -> NodeConfig {
    NodeConfig {
        id: "neox-rs".to_string(),
        name: "neox-rs".to_string(),
        node_type: NodeType::NeoXReth,
        network,
        binary_path: PathBuf::from("/opt/neox/neox-rs"),
        args: Vec::new(),
        runtime_version: "test".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 18332,
        p2p_port: 18333,
        ws_port: None,
        status: NodeStatus::Stopped,
        pid: None,
    }
}

fn body(network: Network) -> toml::Value {
    let text = reth_toml(&node(network), None).expect("neox-rs config renders");
    toml::from_str(&text).expect("neox-rs config is valid TOML")
}

/// Reth's `Config` is `#[serde(default)]` throughout, so a partial `[peers]`
/// table is valid and every unwritten field keeps the client's own default.
#[test]
fn the_peering_table_is_written() {
    let config = body(Network::Mainnet);
    let peers = config["peers"].as_table().expect("[peers] is a table");
    assert_eq!(peers["trusted_nodes_only"].as_bool(), Some(false));
    assert!(peers["connection_info"]["max_outbound"]
        .as_integer()
        .is_some());
}

/// Reth ignores unknown config keys rather than rejecting them, so a `port`
/// written here would read as configured and do nothing at all. Every one of
/// these is a launch flag; none of them is a config key.
#[test]
fn no_runtime_flag_is_ever_written_as_a_config_key() {
    for network in [Network::Mainnet, Network::Testnet, Network::Private] {
        let config = body(network);
        for key in ["port", "rpc", "http", "chain", "datadir"] {
            assert!(
                config.get(key).is_none(),
                "`{key}` is a launch flag, not a {network} config key",
            );
        }
    }
}

/// The file cannot hold the ports, so the header is the only place an operator
/// can see which ones the node will actually listen on.
#[test]
fn the_header_records_the_flags_the_file_cannot_hold() {
    let text = reth_toml(&node(Network::Mainnet), None).expect("renders");
    assert!(text.contains("--chain neox-mainnet"));
    assert!(text.contains("--http.port 18332"));
    assert!(text.contains("--port 18333"));
    assert!(!text.contains("--disable-discovery"));
}

#[test]
fn a_private_network_is_told_it_needs_its_own_chain_spec() {
    let text = reth_toml(&node(Network::Private), None).expect("renders");
    assert!(text.contains("chainspec"), "{text}");
    assert!(text.contains("--disable-discovery"));
    assert!(
        !text.contains("neox-mainnet") && !text.contains("neox-testnet"),
        "a private network must not be pointed at a public chain spec",
    );
}

#[test]
fn the_testnet_header_names_the_testnet_chain() {
    let text = reth_toml(&node(Network::Testnet), None).expect("renders");
    assert!(text.contains("--chain neox-testnet"));
    assert!(!text.contains("neox-mainnet"));
}

#[test]
fn no_neo_n3_setting_leaks_into_the_neox_rs_config() {
    for network in [Network::Mainnet, Network::Testnet, Network::Private] {
        let text = reth_toml(&node(network), None).expect("renders");
        for n3 in [
            "ProtocolConfiguration",
            "StandbyCommittee",
            "seed_nodes",
            "network_magic",
            "seed1.neo.org",
            "860833102",
        ] {
            assert!(!text.contains(n3), "{n3} reached a {network} Neo X config");
        }
    }
}
