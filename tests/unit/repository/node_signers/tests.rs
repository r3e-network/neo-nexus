use std::path::PathBuf;

use super::*;
use crate::signing::SignerKeyRef;
use crate::types::{Network, NewNode, NodeType, StorageEngine};

fn node(repository: &Repository) -> NodeConfig {
    repository
        .create_node(NewNode {
            name: "bound node".to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Testnet,
            binary_path: PathBuf::from("/opt/neo-go"),
            args: Vec::new(),
            runtime_version: "test".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 20332,
            p2p_port: 20333,
            ws_port: None,
        })
        .unwrap()
}

#[test]
fn signer_binding_is_explicit_replaceable_and_removable() {
    let directory = tempfile::tempdir().unwrap();
    let repository = Repository::open(directory.path().join("workspace.db")).unwrap();
    let node = node(&repository);

    assert_eq!(repository.load_node_signer_key(&node.id).unwrap(), None);
    let wallet = SignerKeyRef::new("local-wallet-a", "validator-one").unwrap();
    repository
        .set_node_signer_key(&node.id, Some(&wallet))
        .unwrap();
    assert_eq!(
        repository.load_node_signer_key(&node.id).unwrap(),
        Some(wallet)
    );
    let remote = SignerKeyRef::new("neo-os-prod", "committee-three").unwrap();
    repository
        .set_node_signer_key(&node.id, Some(&remote))
        .unwrap();
    assert_eq!(
        repository.load_node_signer_key(&node.id).unwrap(),
        Some(remote)
    );
    repository.set_node_signer_key(&node.id, None).unwrap();
    assert_eq!(repository.load_node_signer_key(&node.id).unwrap(), None);
}

#[test]
fn active_nodes_cannot_change_signer_identity() {
    let directory = tempfile::tempdir().unwrap();
    let repository = Repository::open(directory.path().join("workspace.db")).unwrap();
    let node = node(&repository);
    repository
        .update_node_status(&node.id, NodeStatus::Running, Some(42))
        .unwrap();

    let error = repository
        .set_node_signer_key(
            &node.id,
            Some(&SignerKeyRef::new("neo-os-prod", "validator").unwrap()),
        )
        .expect_err("an active node must keep its signer identity");
    assert!(error.to_string().contains("stop node"));
}

#[test]
fn signer_binding_rejects_unsafe_profile_ids_and_unknown_nodes() {
    let directory = tempfile::tempdir().unwrap();
    let repository = Repository::open(directory.path().join("workspace.db")).unwrap();
    let _node = node(&repository);

    assert!(SignerKeyRef::new("../fallback", "key").is_err());
    assert!(repository
        .set_node_signer_key(
            "node-missing",
            Some(&SignerKeyRef::new("wallet", "key").unwrap()),
        )
        .is_err());
}
