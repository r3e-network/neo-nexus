use super::*;

#[test]
fn fast_sync_snapshot_manager_rejects_archive_path_traversal() {
    let temp_dir = tempfile::tempdir().unwrap();
    let source = temp_dir.path().join("evil.zip");
    write_zip_snapshot(&source, &[("../escape.txt", b"nope")]);
    let (sha256, _) = sha256_file(&source).unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node_id = create_node(&repo, "neo-rs", NodeType::NeoRs);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();
    let cache = FastSyncSnapshotManager::cache(
        &local_snapshot("evil-zip", NodeType::NeoRs, source.clone(), &sha256),
        temp_dir.path().join("cache"),
    )
    .unwrap();
    let snapshot = FastSyncSnapshot {
        cached_path: Some(cache.path),
        verified_sha256: Some(sha256.clone()),
        verified_at_unix: Some(cache.cached_at_unix),
        download_file_name: Some("evil.zip".to_string()),
        ..local_snapshot("evil-zip", NodeType::NeoRs, source, &sha256)
    };
    let node_data_dir = temp_dir
        .path()
        .join("nodes")
        .join(&node.id)
        .join("data/testnet");

    let error =
        FastSyncSnapshotManager::apply_to_node(&snapshot, &node, &node_data_dir).unwrap_err();

    assert!(error.to_string().contains("unsafe"));
    assert!(!temp_dir.path().join("nodes").join("escape.txt").exists());
}

#[test]
fn fast_sync_snapshot_manager_rejects_archive_import_target_conflicts() {
    let temp_dir = tempfile::tempdir().unwrap();
    let source = temp_dir.path().join("conflict.tar");
    write_tar_snapshot(&source, &[("chain/CURRENT", b"new data")]);
    let (sha256, _) = sha256_file(&source).unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node_id = create_node(&repo, "neo-rs", NodeType::NeoRs);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();
    let cache = FastSyncSnapshotManager::cache(
        &local_snapshot("conflict-tar", NodeType::NeoRs, source.clone(), &sha256),
        temp_dir.path().join("cache"),
    )
    .unwrap();
    let snapshot = FastSyncSnapshot {
        cached_path: Some(cache.path),
        verified_sha256: Some(sha256.clone()),
        verified_at_unix: Some(cache.cached_at_unix),
        download_file_name: Some("conflict.tar".to_string()),
        ..local_snapshot("conflict-tar", NodeType::NeoRs, source, &sha256)
    };
    let node_data_dir = temp_dir
        .path()
        .join("nodes")
        .join(&node.id)
        .join("data/testnet");
    std::fs::create_dir_all(node_data_dir.join("chain")).unwrap();
    std::fs::write(node_data_dir.join("chain/CURRENT"), "existing data").unwrap();

    let error =
        FastSyncSnapshotManager::apply_to_node(&snapshot, &node, &node_data_dir).unwrap_err();

    assert!(error.to_string().contains("already exists"));
    assert_eq!(
        std::fs::read_to_string(node_data_dir.join("chain/CURRENT")).unwrap(),
        "existing data"
    );
}
