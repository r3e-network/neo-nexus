use super::*;

#[test]
fn fast_sync_snapshot_manager_verifies_and_caches_local_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let source = temp_dir.path().join("chain.acc");
    std::fs::write(&source, "neo snapshot bytes").unwrap();
    let (sha256, bytes) = sha256_file(&source).unwrap();
    let snapshot = local_snapshot("testnet-rocksdb", NodeType::NeoRs, source.clone(), &sha256);

    let verification = FastSyncSnapshotManager::verify(&snapshot).unwrap();
    let cache = FastSyncSnapshotManager::cache(&snapshot, temp_dir.path().join("cache")).unwrap();

    assert!(verification.matches);
    assert_eq!(verification.sha256, sha256);
    assert_eq!(verification.bytes, bytes);
    assert_eq!(cache.sha256, verification.sha256);
    assert_eq!(cache.bytes, bytes);
    assert_eq!(
        std::fs::read_to_string(cache.path).unwrap(),
        "neo snapshot bytes"
    );
}

#[test]
fn fast_sync_snapshot_manager_rejects_checksum_mismatch_before_cache_publish() {
    let temp_dir = tempfile::tempdir().unwrap();
    let source = temp_dir.path().join("bad.acc");
    std::fs::write(&source, "unexpected bytes").unwrap();
    let snapshot = local_snapshot(
        "mismatch",
        NodeType::NeoCli,
        source,
        "0000000000000000000000000000000000000000000000000000000000000000",
    );
    let cache_dir = temp_dir.path().join("cache");

    let result = FastSyncSnapshotManager::cache(&snapshot, &cache_dir);

    assert!(result.is_err());
    assert!(std::fs::read_dir(cache_dir).unwrap().next().is_none());
}
