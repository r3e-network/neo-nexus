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

/// Reth's default data directory is under the user's home, not the workspace.
/// Two managed nodes left at that default would share one database and corrupt
/// each other's chain.
#[test]
fn both_clients_are_pinned_to_the_workspace_data_directory() {
    let mut geth = Vec::new();
    geth_args(&mut geth, &work_dir(), PathBuf::from("/cfg/neox.toml"));
    assert_eq!(
        value_of(&geth, "--datadir"),
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
        value_of(&reth, "--datadir"),
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

/// Reth's config file has no port keys, so the ports NeoNexus records for the
/// node only become real if they are passed here.
#[test]
fn the_recorded_ports_reach_the_neox_rs_command_line() {
    let mut args = Vec::new();
    reth_args(
        &node(NodeType::NeoXReth, Network::Mainnet),
        &mut args,
        &work_dir(),
        PathBuf::from("/cfg/neox.toml"),
    );
    assert!(args.iter().any(|arg| arg == "--http"));
    assert_eq!(value_of(&args, "--http.addr"), Some("127.0.0.1"));
    assert_eq!(value_of(&args, "--http.port"), Some("18332"));
    assert_eq!(value_of(&args, "--port"), Some("18333"));
}

/// An operator who typed a flag meant it. Overriding it would leave the node
/// listening somewhere other than where they are watching.
#[test]
fn operator_supplied_flags_are_never_overridden() {
    let mut args = vec![
        "--http.port".to_string(),
        "9999".to_string(),
        "--datadir=/mnt/fast/neox".to_string(),
    ];
    reth_args(
        &node(NodeType::NeoXReth, Network::Mainnet),
        &mut args,
        &work_dir(),
        PathBuf::from("/cfg/neox.toml"),
    );
    assert_eq!(value_of(&args, "--http.port"), Some("9999"));
    assert_eq!(
        args.iter()
            .filter(|arg| arg.starts_with("--datadir"))
            .count(),
        1,
        "the operator's --datadir=… is not joined by a second one"
    );
}

/// A node already carrying its own `--config` keeps it, and NeoNexus reports no
/// managed config rather than writing one the node will not read.
#[test]
fn an_existing_config_flag_suppresses_the_managed_one() {
    for node_type in [NodeType::NeoXGeth, NodeType::NeoXReth] {
        let mut args = vec!["--config".to_string(), "/etc/neox/own.toml".to_string()];
        let managed = match node_type {
            NodeType::NeoXGeth => {
                geth_args(&mut args, &work_dir(), PathBuf::from("/cfg/neox.toml"))
            }
            _ => reth_args(
                &node(node_type, Network::Mainnet),
                &mut args,
                &work_dir(),
                PathBuf::from("/cfg/neox.toml"),
            ),
        };
        assert!(managed.is_none(), "{node_type}");
        assert_eq!(value_of(&args, "--config"), Some("/etc/neox/own.toml"));
    }
}

/// `-c` is not a Neo X flag: Reth declares `config` as `#[arg(long)]` with no
/// short alias, and geth's is `--config`. Treating `-c` as a config flag made
/// the diagnostic promise to leave the operator's file alone while the planner
/// injected `--config` anyway, putting two conflicting flags on one line.
#[test]
fn a_short_c_flag_is_not_mistaken_for_a_neox_config_flag() {
    for node_type in [NodeType::NeoXGeth, NodeType::NeoXReth] {
        let operator_args = vec!["-c".to_string(), "/etc/neox/other.toml".to_string()];
        // The diagnostic reads the operator's own arguments, so it is asked
        // before the planner adds anything to them.
        assert!(
            !crate::launch::runtime_args_include_config(node_type, &operator_args),
            "{node_type}: `-c` must not read as an operator-supplied config",
        );

        let mut args = operator_args;
        let managed = match node_type {
            NodeType::NeoXGeth => {
                geth_args(&mut args, &work_dir(), PathBuf::from("/cfg/neox.toml"))
            }
            _ => reth_args(
                &node(node_type, Network::Mainnet),
                &mut args,
                &work_dir(),
                PathBuf::from("/cfg/neox.toml"),
            ),
        };
        assert!(
            managed.is_some(),
            "{node_type}: the managed config is written"
        );
        // So the planner supplies the managed one, agreeing with what the
        // diagnostic just told the operator would happen.
        assert_eq!(value_of(&args, "--config"), Some("/cfg/neox.toml"));
    }
}

/// Geth's chain id, ports and peers all live in the config file, so its command
/// line stays short — but the file still has to be handed to it.
#[test]
fn geth_is_handed_its_managed_config() {
    let mut args = Vec::new();
    let managed = geth_args(&mut args, &work_dir(), PathBuf::from("/cfg/neox.toml"));
    assert_eq!(managed, Some(PathBuf::from("/cfg/neox.toml")));
    assert_eq!(value_of(&args, "--config"), Some("/cfg/neox.toml"));
    assert!(
        !args.iter().any(|arg| arg == "node"),
        "geth has no node verb"
    );
}
