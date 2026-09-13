use std::path::PathBuf;

use super::*;
use crate::{
    roles::NodeRole,
    signing::{
        ConfiguredSignerBackend, LocalSignerConfig, SignerBackendKind, SignerBackendProfile,
    },
    types::{Network, NewNode, NodeType, StorageEngine},
};

fn repository_node() -> (tempfile::TempDir, Repository, NodeConfig) {
    let directory = tempfile::tempdir().unwrap();
    let repository = Repository::open(directory.path().join("workspace.db")).unwrap();
    let node = repository
        .create_node(NewNode {
            name: "validator".to_string(),
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
        .unwrap();
    (directory, repository, node)
}

#[test]
fn signing_duty_requires_a_complete_node_binding() {
    let (_directory, repository, node) = repository_node();
    repository
        .set_node_role(&node.id, Some(NodeRole::Consensus))
        .unwrap();
    let error = node_signer_key(&repository, &node).unwrap_err();
    assert!(error.to_string().contains("no signer backend and key"));

    let key = SignerKeyRef::new("wallet", "validator-key").unwrap();
    repository
        .set_node_signer_key(&node.id, Some(&key))
        .unwrap();
    assert_eq!(node_signer_key(&repository, &node).unwrap(), Some(key));
}

#[test]
fn read_only_duty_may_remain_unbound() {
    let (_directory, repository, node) = repository_node();
    repository
        .set_node_role(&node.id, Some(NodeRole::Observer))
        .unwrap();
    assert_eq!(node_signer_key(&repository, &node).unwrap(), None);
}

#[test]
fn resolution_refuses_missing_backend_instead_of_using_default() {
    let (_directory, repository, node) = repository_node();
    let key = SignerKeyRef::new("missing", "validator-key").unwrap();
    repository
        .set_node_signer_key(&node.id, Some(&key))
        .unwrap();
    let registry = SignerRegistry::empty();

    let error = resolve_node_signer(&repository, &registry, &node).unwrap_err();
    assert!(error.to_string().contains("unavailable signer backend"));
}

#[test]
fn native_secure_sign_is_not_fictionally_wired_into_neo_go() {
    let (directory, repository, node) = repository_node();
    repository
        .set_node_role(&node.id, Some(NodeRole::Consensus))
        .unwrap();
    let public_key = "031e18532fd4754c02f3041d9c75ceb33b83ffd81ac7ce4fe882ccb1c98bc5896e";
    let key = SignerKeyRef::new("host-signer", public_key).unwrap();
    repository
        .set_node_signer_key(&node.id, Some(&key))
        .unwrap();
    let profile =
        SignerBackendProfile::new("host-signer", "Host signer", SignerBackendKind::LocalSigner)
            .unwrap();
    let backend = ConfiguredSignerBackend::local_signer(
        profile,
        LocalSignerConfig::new("http://127.0.0.1:9991", public_key, 894_710_606).unwrap(),
    )
    .unwrap();
    let registry = SignerRegistry::new([backend], None, None).unwrap();

    let error =
        prepare_node_signer_launch(&repository, Some(&registry), &node, &[], directory.path())
            .unwrap_err();
    assert!(error.to_string().contains("only neo-cli consensus"));
}

#[test]
fn dormant_read_only_binding_does_not_contact_or_fallback_to_custody() {
    let (directory, repository, node) = repository_node();
    repository
        .set_node_role(&node.id, Some(NodeRole::Observer))
        .unwrap();
    repository
        .set_node_signer_key(
            &node.id,
            Some(&SignerKeyRef::new("offline", "key-1").unwrap()),
        )
        .unwrap();

    let launch =
        prepare_node_signer_launch(&repository, None, &node, &[], directory.path()).unwrap();
    assert_eq!(launch.role(), Some(NodeRole::Observer));
    assert!(launch.runtime().is_none());
}

#[test]
fn remote_signer_adapter_is_locked_to_the_reviewed_neo_cli_abi() {
    let (_directory, _repository, mut node) = repository_node();
    node.node_type = NodeType::NeoCli;
    node.runtime_version = "3.10.1".to_string();
    let error = ensure_sign_client_runtime(&node, NodeRole::Consensus).unwrap_err();
    assert!(error.to_string().contains("ABI-locked to neo-cli 3.9.2"));

    node.runtime_version = "v3.9.2".to_string();
    ensure_sign_client_runtime(&node, NodeRole::Consensus).unwrap();
}

#[test]
fn double_signing_hazard_fails_when_another_node_is_running_with_same_signer_key() {
    let (directory, repository, node1) = repository_node();
    let node2 = repository
        .create_node(NewNode {
            name: "validator-backup".to_string(),
            node_type: NodeType::NeoCli,
            network: Network::Testnet,
            binary_path: PathBuf::from("/opt/neo-cli"),
            args: Vec::new(),
            runtime_version: "v3.9.2".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 20334,
            p2p_port: 20335,
            ws_port: None,
        })
        .unwrap();

    let public_key = "031e18532fd4754c02f3041d9c75ceb33b83ffd81ac7ce4fe882ccb1c98bc5896e";
    let key = SignerKeyRef::new("host-signer", public_key).unwrap();
    let profile =
        SignerBackendProfile::new("host-signer", "Host signer", SignerBackendKind::LocalSigner)
            .unwrap();
    let backend = ConfiguredSignerBackend::local_signer(
        profile,
        LocalSignerConfig::new("http://127.0.0.1:9991", public_key, 894_710_606).unwrap(),
    )
    .unwrap();
    let registry = SignerRegistry::new([backend], None, None).unwrap();

    repository
        .set_node_role(&node1.id, Some(NodeRole::Consensus))
        .unwrap();
    repository
        .set_node_signer_key(&node1.id, Some(&key))
        .unwrap();

    repository
        .set_node_role(&node2.id, Some(NodeRole::Consensus))
        .unwrap();

    // The database itself now refuses a second claim on one key, so even a
    // caller writing raw SQL cannot reach the state this test needs.
    let conn = rusqlite::Connection::open(directory.path().join("workspace.db")).unwrap();
    let refused = conn.execute(
        "INSERT INTO node_signer_bindings (node_id, backend_id, key_id) VALUES (?1, ?2, ?3)",
        rusqlite::params![node2.id, key.backend_id, key.key_id],
    );
    assert!(
        refused.is_err(),
        "the exclusive-lease index let a second instance claim one key"
    );

    // The runtime guard is the layer beneath that index, and it still has to
    // hold for a workspace that reached the forbidden state before the index
    // existed. Drop the index to be that workspace.
    conn.execute("DROP INDEX idx_node_signer_bindings_exclusive_lease", [])
        .unwrap();
    conn.execute(
        "INSERT INTO node_signer_bindings (node_id, backend_id, key_id) VALUES (?1, ?2, ?3)",
        rusqlite::params![node2.id, key.backend_id, key.key_id],
    )
    .unwrap();

    // Mark node1 as actively running with a PID
    repository
        .update_node_status(&node1.id, crate::types::NodeStatus::Running, Some(4242))
        .unwrap();

    // Launching node2 with the same key must fail with double-signing hazard
    let err =
        prepare_node_signer_launch(&repository, Some(&registry), &node2, &[], directory.path())
            .unwrap_err();
    assert!(
        err.to_string().contains("double-signing hazard"),
        "expected double-signing hazard, got: {err}"
    );

    // If node1 is stopped, node2 can proceed past the double-signing barrier
    repository
        .update_node_status(&node1.id, crate::types::NodeStatus::Stopped, None)
        .unwrap();
    let res =
        prepare_node_signer_launch(&repository, Some(&registry), &node2, &[], directory.path());
    if let Err(e) = res {
        assert!(
            !e.to_string().contains("double-signing hazard"),
            "should not fail with double-signing hazard when node1 is stopped"
        );
    }
}
