//! Release transaction orchestration: one durable, rollback-able multi-file
//! upgrade. The acceptance gate is injected so the rollback path is tested
//! deterministically without a real chain binary.

use std::path::{Path, PathBuf};

use neo_nexus::{
    release_transaction::{apply_release_transaction, TargetRelease},
    repository::Repository,
    runtime::RuntimeInstallation,
    types::{Network, NewNode, NodeType, StorageEngine},
};

fn workspace() -> (tempfile::TempDir, Repository, neo_nexus::types::NodeConfig) {
    let home = tempfile::tempdir().expect("tempdir");
    let repository = Repository::open(home.path().join("neonexus.db")).unwrap();
    let node_id = repository
        .create_node(NewNode {
            name: "release-node".to_string(),
            node_type: NodeType::NeoRs,
            network: Network::Testnet,
            binary_path: PathBuf::from("/not/a/real/binary"),
            args: Vec::new(),
            runtime_version: "1.0.0".to_string(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port: 20332,
            p2p_port: 20333,
            ws_port: None,
        })
        .unwrap()
        .id;
    let node = repository
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|candidate| candidate.id == node_id)
        .unwrap();
    (home, repository, node)
}

fn record_installation(repository: &Repository, binary_path: &Path, version: &str) {
    let (sha, bytes) = neo_nexus::snapshots::sha256_file(binary_path).expect("fixture sha256");
    repository
        .upsert_runtime_installation(&RuntimeInstallation {
            package_id: format!("pkg-{version}"),
            label: version.to_string(),
            node_type: NodeType::NeoRs,
            version: version.to_string(),
            platform: neo_nexus::runtime::RuntimePlatform::current(),
            binary_path: binary_path.to_path_buf(),
            sha256: sha,
            signature_verified: false,
            signer_public_key: None,
            bytes,
            installed_at_unix: 1,
        })
        .expect("record fixture installation");
}

fn target(home: &tempfile::TempDir, repository: &Repository) -> TargetRelease {
    let path = home.path().join("runtimes").join("neo-node-v2");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, b"fixture runtime").unwrap();
    record_installation(repository, &path, "2.0.0");
    TargetRelease {
        version: "2.0.0".into(),
        binary_path: path,
    }
}

#[test]
fn a_successful_release_commits_runtime_and_records_no_pending_transaction() {
    let (home, repository, node) = workspace();
    let target = target(&home, &repository);
    let message = apply_release_transaction(
        &repository,
        home.path(),
        &node,
        &target,
        Some(Box::new(|_| Ok(()))),
    )
    .expect("release applies");
    assert!(message.contains("committed"));
    let updated = repository
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|candidate| candidate.id == node.id)
        .unwrap();
    assert_eq!(updated.runtime_version, "2.0.0");
    assert!(repository
        .list_pending_release_transactions()
        .unwrap()
        .is_empty());
}

#[test]
fn a_failed_acceptance_rolls_back_config_and_keeps_the_old_version() {
    let (home, repository, node) = workspace();
    let target = target(&home, &repository);
    let error = apply_release_transaction(
        &repository,
        home.path(),
        &node,
        &target,
        Some(Box::new(|_| Err(anyhow::anyhow!("smoke failed")))),
    )
    .expect_err("acceptance failure must fail the release");
    assert!(error.to_string().contains("rolled back"));
    let updated = repository
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|candidate| candidate.id == node.id)
        .unwrap();
    assert_eq!(updated.runtime_version, "1.0.0");
    assert!(repository
        .list_pending_release_transactions()
        .unwrap()
        .is_empty());
}

#[test]
fn a_release_phase_is_cas_guarded() {
    let (_home, repository, _node) = workspace();
    let record = repository
        .begin_release_transaction("node-1", "1", "/old", "2", "/new", "/backup", 10)
        .unwrap();
    assert!(!repository
        .advance_release_transaction(&record.id, "applied", "accepted", None, 11)
        .unwrap());
    assert!(repository
        .advance_release_transaction(&record.id, "requested", "backing-up", None, 12)
        .unwrap());
}
