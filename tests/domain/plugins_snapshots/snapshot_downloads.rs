use super::*;

#[test]
fn fast_sync_snapshot_download_cache_publishes_only_matching_https_hash() {
    let temp_dir = tempfile::tempdir().unwrap();
    let bytes = b"remote snapshot bytes";
    let source = temp_dir.path().join("remote.acc");
    std::fs::write(&source, bytes).unwrap();
    let (sha256, expected_bytes) = sha256_file(&source).unwrap();
    let request = SnapshotDownloadRequest {
        snapshot_id: "neo-rs-remote".to_string(),
        url: "https://snapshots.example.com/neo-rs-testnet.acc".to_string(),
        file_name: "neo-rs-testnet.acc".to_string(),
        expected_sha256: sha256.clone(),
        max_bytes: 1024,
    };

    let cache = FastSyncSnapshotManager::cache_download_from_reader(
        &request,
        temp_dir.path().join("cache"),
        Cursor::new(bytes),
    )
    .unwrap();

    assert_eq!(cache.sha256, sha256);
    assert_eq!(cache.bytes, expected_bytes);
    assert!(cache.path.ends_with(format!(
        "fastsync-neo-rs-remote-{}.acc",
        &cache.sha256[..12]
    )));
    assert_eq!(std::fs::read(&cache.path).unwrap(), bytes);
}

#[test]
fn fast_sync_snapshot_download_rejects_insecure_urls_and_downgrade_redirects() {
    let request = SnapshotDownloadRequest {
        snapshot_id: "neo-rs-remote".to_string(),
        url: "http://snapshots.example.com/neo-rs-testnet.acc".to_string(),
        file_name: "neo-rs-testnet.acc".to_string(),
        expected_sha256: "a".repeat(64),
        max_bytes: 1024,
    };
    let current = url::Url::parse("https://snapshots.example.com/files/chain.acc").unwrap();

    let error = validate_snapshot_download_request(&request).unwrap_err();
    let redirect = validate_snapshot_https_redirect(&current, "http://evil.example.com/chain.acc");

    assert!(error.to_string().contains("must use HTTPS"));
    assert!(redirect
        .unwrap_err()
        .to_string()
        .contains("must stay on HTTPS"));
}

#[test]
fn fast_sync_snapshot_download_rejects_hash_mismatch_without_publish() {
    let temp_dir = tempfile::tempdir().unwrap();
    let request = SnapshotDownloadRequest {
        snapshot_id: "neo-rs-mismatch".to_string(),
        url: "https://snapshots.example.com/neo-rs-testnet.acc".to_string(),
        file_name: "neo-rs-testnet.acc".to_string(),
        expected_sha256: "0".repeat(64),
        max_bytes: 1024,
    };
    let cache_dir = temp_dir.path().join("cache");

    let error = FastSyncSnapshotManager::cache_download_from_reader(
        &request,
        &cache_dir,
        Cursor::new(b"real snapshot"),
    )
    .unwrap_err();

    assert!(error.to_string().contains("checksum mismatch"));
    assert!(std::fs::read_dir(cache_dir).unwrap().next().is_none());
}
