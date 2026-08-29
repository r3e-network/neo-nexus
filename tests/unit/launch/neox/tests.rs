use std::path::PathBuf;

use super::*;
use crate::types::{NodeStatus, NodeType, StorageEngine};

fn node(node_type: NodeType, network: Network) -> NodeConfig {
    NodeConfig {
        id: "neox".to_string(),
        name: "neox".to_string(),
        node_type,
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

fn work_dir() -> PathBuf {
    PathBuf::from("/workspace/nodes/neox")
}

/// The value that follows a flag, so a test asserts on the pairing rather than
/// on the position of a string in a flat list.
fn value_of<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}

/// The data directory as a launch argument, with host separators folded to '/'
/// so one expectation holds on Windows and Unix alike.
fn pinned_datadir(args: &[String]) -> Option<String> {
    value_of(args, "--datadir").map(|value| value.replace(std::path::MAIN_SEPARATOR, "/"))
}

/// Reth's default data directory is under the user's home, not the workspace.
/// Two managed nodes left at that default would share one database and corrupt
/// each other's chain.
#[test]
fn both_clients_are_pinned_to_the_workspace_data_directory() {
    let mut geth = Vec::new();
    geth_args(&mut geth, &work_dir(), PathBuf::from("/cfg/neox.toml"));
    assert_eq!(
        pinned_datadir(&geth).as_deref(),
        Some("/workspace/nodes/neox/data")
    );

    let mut reth = Vec::new();
    reth_args(
        &node(NodeType::NeoXReth, Network::Mainnet),
        &mut reth,
        &work_dir(),
        PathBuf::from("/cfg/neox.toml"),
    );
    assert_eq!(
        pinned_datadir(&reth).as_deref(),
        Some("/workspace/nodes/neox/data")
    );
}

/// Without `--chain`, neox-rs starts on Ethereum mainnet: the config file has
/// no chain key at all, so the node would look managed and be on the wrong
/// network entirely.
#[test]
fn neox_rs_is_put_on_the_neo_x_chain_by_flag() {
    for (network, expected) in [
        (Network::Mainnet, "neox-mainnet"),
        (Network::Testnet, "neox-testnet"),
    ] {
        let mut args = Vec::new();
        reth_args(
            &node(NodeType::NeoXReth, network),
            &mut args,
            &work_dir(),
            PathBuf::from("/cfg/neox.toml"),
        );
        assert_eq!(args.first().map(String::as_str), Some("node"));
        assert_eq!(value_of(&args, "--chain"), Some(expected));
    }
}

/// A private Neo X network has no built-in chain spec. Passing a made-up name
/// aborts at startup; passing a public one puts the node on a public chain.
#[test]
fn a_private_network_gets_no_invented_chain_and_no_discovery() {
    let mut args = Vec::new();
    reth_args(
        &node(NodeType::NeoXReth, Network::Private),
        &mut args,
        &work_dir(),
        PathBuf::from("/cfg/neox.toml"),
    );
    assert!(!args.iter().any(|arg| arg == "--chain"));
    assert!(args.iter().any(|arg| arg == "--disable-discovery"));
}

#[test]
fn the_public_networks_keep_discovery_on() {
    for network in [Network::Mainnet, Network::Testnet] {
        let mut args = Vec::new();
        reth_args(
            &node(NodeType::NeoXReth, network),
            &mut args,
            &work_dir(),
            PathBuf::from("/cfg/neox.toml"),
        );
        assert!(!args.iter().any(|arg| arg == "--disable-discovery"));
    }
}

#[path = "flags.rs"]
mod flags;
