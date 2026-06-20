use super::*;

#[test]
fn repository_persists_fast_sync_snapshot_manifest_and_cache_state() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let source = temp_dir.path().join("snapshot.acc");
    std::fs::write(&source, "cached bytes").unwrap();
    let (sha256, _) = sha256_file(&source).unwrap();

    let snapshot = repo
        .upsert_fast_sync_snapshot(NewFastSyncSnapshot {
            id: "neo-rs-testnet".to_string(),
            label: "Neo RS Testnet".to_string(),
            network: Network::Testnet,
            node_type: NodeType::NeoRs,
            source_path: source,
            source_url: Some("https://snapshots.example.com/neo-rs-testnet.acc".to_string()),
            download_file_name: Some("neo-rs-testnet.acc".to_string()),
            download_max_bytes: 1024 * 1024,
            expected_sha256: sha256.clone(),
        })
        .unwrap();
    let verification = FastSyncSnapshotManager::verify(&snapshot).unwrap();
    let cache = FastSyncSnapshotManager::cache(&snapshot, temp_dir.path().join("cache")).unwrap();

    repo.mark_fast_sync_snapshot_verified(&snapshot.id, &verification)
        .unwrap();
    repo.mark_fast_sync_snapshot_cached(&snapshot.id, &cache)
        .unwrap();

    let snapshots = repo.list_fast_sync_snapshots().unwrap();
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].id, "neo-rs-testnet");
    assert_eq!(
        snapshots[0].source_url.as_deref(),
        Some("https://snapshots.example.com/neo-rs-testnet.acc")
    );
    assert_eq!(
        snapshots[0].download_file_name.as_deref(),
        Some("neo-rs-testnet.acc")
    );
    assert_eq!(snapshots[0].download_max_bytes, 1024 * 1024);
    assert_eq!(snapshots[0].expected_sha256, sha256);
    assert_eq!(
        snapshots[0].verified_sha256.as_deref(),
        Some(sha256.as_str())
    );
    assert_eq!(snapshots[0].cached_path.as_ref(), Some(&cache.path));
    assert_eq!(snapshots[0].bytes, Some(cache.bytes));
}
