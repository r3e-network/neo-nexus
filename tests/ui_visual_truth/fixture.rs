//! A seeded workspace so the rendered previews show a realistic operator fleet
//! instead of a set of empty states.

use std::path::{Path, PathBuf};

use neo_nexus::{
    repository::Repository,
    types::{Network, NewNode, NodeType, StorageEngine},
};

/// One demo node definition: name, runtime, network, and its port triple.
struct DemoNode {
    name: &'static str,
    node_type: NodeType,
    network: Network,
    runtime_version: &'static str,
    rpc_port: u16,
    p2p_port: u16,
}

const DEMO_NODES: [DemoNode; 7] = [
    DemoNode {
        name: "mainnet-consensus-01",
        node_type: NodeType::NeoCli,
        network: Network::Mainnet,
        runtime_version: "3.7.4",
        rpc_port: 10332,
        p2p_port: 10333,
    },
    DemoNode {
        name: "mainnet-rpc-edge",
        node_type: NodeType::NeoGo,
        network: Network::Mainnet,
        runtime_version: "0.106.3",
        rpc_port: 10334,
        p2p_port: 10335,
    },
    DemoNode {
        name: "testnet-oracle",
        node_type: NodeType::NeoCli,
        network: Network::Testnet,
        runtime_version: "3.7.4",
        rpc_port: 20332,
        p2p_port: 20333,
    },
    DemoNode {
        name: "testnet-neofs-gate",
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        runtime_version: "0.4.1",
        rpc_port: 20334,
        p2p_port: 20335,
    },
    // One node per Neo X client, so every surface is rendered with both chain
    // families present rather than only the Neo N3 one.
    DemoNode {
        name: "neox-mainnet-rpc",
        node_type: NodeType::NeoXGeth,
        network: Network::Mainnet,
        runtime_version: "1.2.0",
        rpc_port: 18332,
        p2p_port: 18333,
    },
    DemoNode {
        name: "neox-testnet-validator",
        node_type: NodeType::NeoXReth,
        network: Network::Testnet,
        runtime_version: "0.9.2",
        rpc_port: 18334,
        p2p_port: 18335,
    },
    DemoNode {
        name: "private-committee-a",
        node_type: NodeType::NeoGo,
        network: Network::Private,
        runtime_version: "0.106.3",
        rpc_port: 30332,
        p2p_port: 30333,
    },
];

/// Opens a workspace at `path`, seeds it with the demo fleet and the given UI
/// preferences, and returns a repository handle ready to hand to the app.
pub fn seeded_workspace(path: &Path, dark: bool, view_key: &str) -> Repository {
    seeded_workspace_at(path, dark, view_key, &[])
}

/// As [`seeded_workspace`], but also pins sub-tabs so a preview can land on a
/// page that is not its section's default.
pub fn seeded_workspace_at(
    path: &Path,
    dark: bool,
    view_key: &str,
    sections: &[(&str, &str)],
) -> Repository {
    let repository = Repository::open(path).unwrap();
    for (key, value) in sections {
        repository.save_workspace_section(key, value).unwrap();
    }
    repository.save_app_dark_mode(dark).unwrap();
    repository.save_app_inspector_visible(true).unwrap();
    repository.save_workspace_last_view(view_key).unwrap();
    if repository.list_nodes().unwrap().is_empty() {
        for node in DEMO_NODES {
            repository.create_node(new_node(&node)).unwrap();
        }
    }
    drop(repository);
    Repository::open(path).unwrap()
}

fn new_node(node: &DemoNode) -> NewNode {
    NewNode {
        name: node.name.to_string(),
        node_type: node.node_type,
        network: node.network,
        binary_path: binary_path(node.node_type),
        args: Vec::new(),
        runtime_version: node.runtime_version.to_string(),
        storage_engine: storage_engine(node.node_type),
        rpc_port: node.rpc_port,
        p2p_port: node.p2p_port,
        ws_port: Some(node.rpc_port + 100),
    }
}

fn binary_path(node_type: NodeType) -> PathBuf {
    PathBuf::from(match node_type {
        NodeType::NeoCli => "/opt/neo/neo-cli/neo-cli",
        NodeType::NeoGo => "/opt/neo/neo-go/bin/neo-go",
        NodeType::NeoRs => "/opt/neo/neo-rs/bin/neo-rs",
        NodeType::NeoXGeth => "/opt/neox/geth/bin/geth",
        NodeType::NeoXReth => "/opt/neox/neox-rs/bin/neox-rs",
    })
}

fn storage_engine(node_type: NodeType) -> StorageEngine {
    node_type.default_storage_engine()
}
