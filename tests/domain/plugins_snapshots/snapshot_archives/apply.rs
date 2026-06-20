use super::*;

#[test]
fn fast_sync_snapshot_manager_applies_cached_snapshot_to_node_data_dir() {
    let temp_dir = tempfile::tempdir().unwrap();
    let source = temp_dir.path().join("chain.acc");
    std::fs::write(&source, "node data package").unwrap();
    let (sha256, bytes) = sha256_file(&source).unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node_id = create_node(&repo, "neo-rs", NodeType::NeoRs);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();
    let cache = FastSyncSnapshotManager::cache(
        &local_snapshot("testnet-rocksdb", NodeType::NeoRs, source.clone(), &sha256),
        temp_dir.path().join("cache"),
    )
    .unwrap();
    let snapshot = FastSyncSnapshot {
        cached_path: Some(cache.path),
        verified_sha256: Some(sha256.clone()),
        verified_at_unix: Some(cache.cached_at_unix),
        bytes: Some(bytes),
        ..local_snapshot("testnet-rocksdb", NodeType::NeoRs, source, &sha256)
    };

    let application = FastSyncSnapshotManager::apply_to_node(
        &snapshot,
        &node,
        temp_dir
            .path()
            .join("nodes")
            .join(&node.id)
            .join("data/testnet"),
    )
    .unwrap();

    assert_eq!(application.sha256, sha256);
    assert_eq!(application.bytes, bytes);
    assert_eq!(application.import_mode, SnapshotImportMode::RawFile);
    assert_eq!(application.imported_files, 1);
    assert_eq!(application.expanded_bytes, bytes);
    assert_eq!(
        std::fs::read_to_string(&application.snapshot_path).unwrap(),
        "node data package"
    );
    let manifest = std::fs::read_to_string(&application.manifest_path).unwrap();
    assert!(manifest.contains("\"snapshot_id\": \"testnet-rocksdb\""));
    assert!(manifest.contains(&node.id));
}
