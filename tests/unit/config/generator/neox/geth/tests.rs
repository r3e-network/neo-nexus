use std::path::PathBuf;

use super::*;
use crate::types::{NodeStatus, NodeType, StorageEngine};

fn node(network: Network) -> NodeConfig {
    NodeConfig {
        id: "neox-geth".to_string(),
        name: "neox-geth".to_string(),
        node_type: NodeType::NeoXGeth,
        network,
        binary_path: PathBuf::from("/opt/neox/geth"),
        args: Vec::new(),
        runtime_version: "test".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 18332,
        p2p_port: 18333,
        ws_port: Some(18334),
        status: NodeStatus::Stopped,
        pid: None,
    }
}

fn rendered(network: Network) -> toml::Value {
    let text = geth_toml(&node(network), None).expect("geth config renders");
    toml::from_str(&text).expect("geth config is valid TOML")
}

/// Geth has no Neo X network preset, so `NetworkId` plus a datadir initialised
/// from the published genesis is the only thing putting a node on Neo X.
#[test]
fn the_chain_id_is_the_published_neo_x_one() {
    assert_eq!(
        rendered(Network::Mainnet)["Eth"]["NetworkId"].as_integer(),
        Some(47_763)
    );
    assert_eq!(
        rendered(Network::Testnet)["Eth"]["NetworkId"].as_integer(),
        Some(12_227_332)
    );
}

/// `DataDir = ""` is not an unset field in geth: it starts an ephemeral
/// in-memory node whose chain is discarded when the process exits.
#[test]
fn the_data_directory_is_left_to_the_launch_flag() {
    assert!(
        rendered(Network::Mainnet)["Node"].get("DataDir").is_none(),
        "an empty DataDir would silently start an ephemeral node"
    );
}

#[test]
fn the_recorded_ports_are_written_into_the_config() {
    let config = rendered(Network::Mainnet);
    assert_eq!(config["Node"]["HTTPPort"].as_integer(), Some(18_332));
    assert_eq!(config["Node"]["WSPort"].as_integer(), Some(18_334));
    assert_eq!(config["Node"]["P2P"]["ListenAddr"].as_str(), Some(":18333"));
}

/// The RPC listener binds to loopback, and the namespaces that can rewrite
/// chain state or unlock accounts are never exposed.
#[test]
fn the_rpc_surface_is_loopback_and_carries_no_privileged_namespace() {
    let config = rendered(Network::Mainnet);
    assert_eq!(config["Node"]["HTTPHost"].as_str(), Some("127.0.0.1"));
    assert_eq!(config["Node"]["WSHost"].as_str(), Some("127.0.0.1"));

    let modules: Vec<&str> = config["Node"]["HTTPModules"]
        .as_array()
        .expect("HTTPModules is an array")
        .iter()
        .filter_map(toml::Value::as_str)
        .collect();
    assert!(modules.contains(&"eth"));
    for privileged in ["admin", "debug", "personal", "miner", "txpool_setGasPrice"] {
        assert!(!modules.contains(&privileged), "{privileged} is exposed");
    }
}

/// A non-empty `WSHost` is what starts geth's WebSocket server. Writing one
/// for a node with no WebSocket port would open a listener on geth's default
/// 8546 — a port NeoNexus never reserved and another node may already hold.
#[test]
fn no_websocket_listener_is_opened_for_a_node_without_a_websocket_port() {
    let mut without = node(Network::Mainnet);
    without.ws_port = None;
    let text = geth_toml(&without, None).expect("renders");
    let config: toml::Value = toml::from_str(&text).unwrap();

    assert!(config["Node"].get("WSHost").is_none());
    assert!(config["Node"].get("WSPort").is_none());
    assert!(config["Node"].get("WSModules").is_none());
    // The HTTP surface is unaffected: only the WebSocket one is withheld.
    assert_eq!(config["Node"]["HTTPPort"].as_integer(), Some(18_332));
}

#[test]
fn the_public_bootnodes_are_written_and_a_private_network_gets_none() {
    let public = rendered(Network::Mainnet);
    let peers = public["Node"]["P2P"]["BootstrapNodes"]
        .as_array()
        .expect("BootstrapNodes is an array");
    assert_eq!(peers.len(), 2);
    for peer in peers {
        assert!(peer.as_str().is_some_and(|url| url.starts_with("enode://")));
    }

    let private = rendered(Network::Private);
    assert!(private["Node"]["P2P"]["BootstrapNodes"]
        .as_array()
        .is_some_and(Vec::is_empty));
    assert_eq!(
        private["Node"]["P2P"]["NoDiscovery"].as_bool(),
        Some(true),
        "a private network has nothing to discover"
    );
}

/// No Neo N3 setting may reach an EVM config: an N3 magic taken as a chain id
/// would produce signatures replayable nowhere.
#[test]
fn no_neo_n3_setting_leaks_into_the_geth_config() {
    for network in [Network::Mainnet, Network::Testnet, Network::Private] {
        let text = geth_toml(&node(network), None).expect("renders");
        for n3 in [
            "ProtocolConfiguration",
            "StandbyCommittee",
            "SeedList",
            "seed1.neo.org",
            "860833102",
            "894710606",
        ] {
            assert!(!text.contains(n3), "{n3} reached a {network} Neo X config");
        }
    }
}

/// The operator has to run `geth init` themselves — NeoNexus cannot invent a
/// genesis allocation — so the file has to say so where they will read it.
#[test]
fn the_header_names_the_genesis_step() {
    let mainnet = geth_toml(&node(Network::Mainnet), None).expect("renders");
    assert!(mainnet.contains("geth init"));
    assert!(mainnet.contains("genesis_mainnet.json"));

    let private = geth_toml(&node(Network::Private), None).expect("renders");
    assert!(private.contains("private network"));
    assert!(!private.contains("genesis_mainnet.json"));
}
