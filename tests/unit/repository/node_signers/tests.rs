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

#[test]
fn signer_key_cannot_be_claimed_by_multiple_nodes_iam_isolation() {
    let directory = tempfile::tempdir().unwrap();
    let repository = Repository::open(directory.path().join("workspace.db")).unwrap();
    let node1 = repository
        .create_node(NewNode {
            name: "node-1".to_string(),
            node_type: NodeType::NeoCli,
            network: Network::Mainnet,
            binary_path: PathBuf::from("/opt/neo/neo-cli"),
            args: Vec::new(),
            runtime_version: "latest".to_string(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port: 10332,
            p2p_port: 20333,
            ws_port: None,
        })
        .unwrap();
    let node2 = repository
        .create_node(NewNode {
            name: "node-2".to_string(),
            node_type: NodeType::NeoCli,
            network: Network::Mainnet,
            binary_path: PathBuf::from("/opt/neo/neo-cli"),
            args: Vec::new(),
            runtime_version: "latest".to_string(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port: 10334,
            p2p_port: 20335,
            ws_port: None,
        })
        .unwrap();

    let key = SignerKeyRef::new("wallet-profile", "consensus-key-01").unwrap();
    repository.set_node_signer_key(&node1.id, Some(&key)).unwrap();

    let err = repository
        .set_node_signer_key(&node2.id, Some(&key))
        .expect_err("cross-node signer key lease must be forbidden");
    assert!(
        err.to_string().contains("IAM Isolation Violation"),
        "error message should cite IAM isolation: {err}"
    );

    // After node1 releases the lease, node2 can acquire it cleanly
    repository.set_node_signer_key(&node1.id, None).unwrap();
    assert_eq!(repository.load_node_signer_key(&node1.id).unwrap(), None);

    repository.set_node_signer_key(&node2.id, Some(&key)).unwrap();
    assert_eq!(
        repository.load_node_signer_key(&node2.id).unwrap(),
        Some(key)
    );
}

#[test]
fn find_node_by_signer_key_and_list_all_bindings() {
    let directory = tempfile::tempdir().unwrap();
    let repository = Repository::open(directory.path().join("workspace.db")).unwrap();
    let node1 = repository
        .create_node(NewNode {
            name: "validator-alpha".to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Mainnet,
            binary_path: PathBuf::from("/opt/neo-go"),
            args: Vec::new(),
            runtime_version: "test".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 10332,
            p2p_port: 10333,
            ws_port: None,
        })
        .unwrap();

    let node2 = repository
        .create_node(NewNode {
            name: "validator-beta".to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Mainnet,
            binary_path: PathBuf::from("/opt/neo-go"),
            args: Vec::new(),
            runtime_version: "test".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 10334,
            p2p_port: 10335,
            ws_port: None,
        })
        .unwrap();

    let key1 = SignerKeyRef::new("vault-primary", "key-alpha").unwrap();
    let key2 = SignerKeyRef::new("vault-primary", "key-beta").unwrap();

    repository.set_node_signer_key(&node1.id, Some(&key1)).unwrap();
    repository.set_node_signer_key(&node2.id, Some(&key2)).unwrap();

    // Verify reverse lookup
    assert_eq!(
        repository.find_node_by_signer_key("vault-primary", "key-alpha").unwrap(),
        Some(node1.id.clone())
    );
    assert_eq!(
        repository.find_node_by_signer_key("vault-primary", "key-beta").unwrap(),
        Some(node2.id.clone())
    );
    assert_eq!(
        repository.find_node_by_signer_key("vault-primary", "key-unbound").unwrap(),
        None
    );

    // Verify listing
    let all = repository.list_all_signer_bindings().unwrap();
    assert_eq!(all.len(), 2);
    assert!(all.iter().any(|(nid, k)| nid == &node1.id && k == &key1));
    assert!(all.iter().any(|(nid, k)| nid == &node2.id && k == &key2));
}

