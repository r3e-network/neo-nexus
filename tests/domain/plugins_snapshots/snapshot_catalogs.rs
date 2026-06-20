use super::*;

#[test]
fn fast_sync_snapshot_catalog_parses_entries_and_converts_to_manifest() {
    let catalog_text = serde_json::json!({
        "schema_version": 1,
        "generated_at_unix": 1_800_000_400u64,
        "snapshots": [
            {
                "id": "neo-rs-testnet-rocksdb",
                "label": "neo-rs Testnet RocksDB",
                "network": "testnet",
                "node_type": "neo-rs",
                "source_url": "https://snapshots.example.com/neo-rs-testnet.acc",
                "download_file_name": "neo-rs-testnet.acc",
                "expected_sha256": "b".repeat(64),
                "max_bytes": 4096
            },
            {
                "id": "neo-cli-mainnet-leveldb",
                "label": "neo-cli Mainnet LevelDB",
                "network": "mainnet",
                "node_type": "neo-cli",
                "url": "https://snapshots.example.com/neo-cli-mainnet.acc",
                "file_name": "neo-cli-mainnet.acc",
                "expected_sha256": "c".repeat(64)
            }
        ]
    })
    .to_string();

    let catalog = FastSyncSnapshotCatalog::from_json(&catalog_text).unwrap();
    let compatible = catalog.compatible_entries(Network::Testnet, NodeType::NeoRs);
    let manifest = compatible[0].to_new_snapshot();

    assert_eq!(catalog.snapshots.len(), 2);
    assert_eq!(compatible.len(), 1);
    assert_eq!(manifest.id, "neo-rs-testnet-rocksdb");
    assert_eq!(
        manifest.source_url.as_deref(),
        Some("https://snapshots.example.com/neo-rs-testnet.acc")
    );
    assert_eq!(
        manifest.download_file_name.as_deref(),
        Some("neo-rs-testnet.acc")
    );
    assert_eq!(manifest.download_max_bytes, 4096);
    assert_eq!(
        catalog.snapshots[1].max_bytes,
        FastSyncSnapshotManager::DEFAULT_DOWNLOAD_MAX_BYTES
    );
}

#[test]
fn fast_sync_snapshot_catalog_loads_signed_local_catalog() {
    let temp_dir = tempfile::tempdir().unwrap();
    let catalog_text = serde_json::json!({
        "schema_version": 1,
        "snapshots": [
            {
                "id": "neo-rs-signed-snapshot",
                "label": "neo-rs signed snapshot",
                "network": "testnet",
                "node_type": "neo-rs",
                "url": "https://snapshots.example.com/neo-rs-testnet.acc",
                "file_name": "neo-rs-testnet.acc",
                "expected_sha256": "d".repeat(64)
            }
        ]
    })
    .to_string();
    let catalog_path = temp_dir.path().join("snapshot-catalog.json");
    let signature_path = temp_dir.path().join("snapshot-catalog.json.sig");
    let signing_key = SigningKey::from_bytes(&[41u8; 32]);
    let signature = signing_key.sign(catalog_text.as_bytes());
    std::fs::write(&catalog_path, catalog_text).unwrap();
    std::fs::write(
        &signature_path,
        BASE64_STANDARD.encode(signature.to_bytes()),
    )
    .unwrap();

    let load = FastSyncSnapshotManager::load_catalog(&SnapshotCatalogLoadRequest {
        source: catalog_path.display().to_string(),
        signature_source: Some(signature_path.display().to_string()),
        ed25519_public_key: Some(BASE64_STANDARD.encode(signing_key.verifying_key().to_bytes())),
        max_bytes: FastSyncSnapshotManager::DEFAULT_CATALOG_MAX_BYTES,
    })
    .unwrap();

    assert_eq!(load.signature_verified, Some(true));
    assert_eq!(load.catalog.snapshots.len(), 1);
    assert_eq!(load.catalog.snapshots[0].id, "neo-rs-signed-snapshot");
}

#[test]
fn fast_sync_snapshot_catalog_rejects_tampered_signature() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_catalog = serde_json::json!({
        "schema_version": 1,
        "snapshots": [
            {
                "id": "neo-rs-original-snapshot",
                "label": "neo-rs original snapshot",
                "network": "testnet",
                "node_type": "neo-rs",
                "url": "https://snapshots.example.com/neo-rs-testnet.acc",
                "file_name": "neo-rs-testnet.acc",
                "expected_sha256": "e".repeat(64)
            }
        ]
    })
    .to_string();
    let tampered_catalog = original_catalog.replace("neo-rs-original", "neo-rs-tampered");
    let catalog_path = temp_dir.path().join("snapshot-catalog.json");
    let signature_path = temp_dir.path().join("snapshot-catalog.json.sig");
    let signing_key = SigningKey::from_bytes(&[43u8; 32]);
    let signature = signing_key.sign(original_catalog.as_bytes());
    std::fs::write(&catalog_path, tampered_catalog).unwrap();
    std::fs::write(
        &signature_path,
        BASE64_STANDARD.encode(signature.to_bytes()),
    )
    .unwrap();

    let error = FastSyncSnapshotManager::load_catalog(&SnapshotCatalogLoadRequest {
        source: catalog_path.display().to_string(),
        signature_source: Some(signature_path.display().to_string()),
        ed25519_public_key: Some(BASE64_STANDARD.encode(signing_key.verifying_key().to_bytes())),
        max_bytes: FastSyncSnapshotManager::DEFAULT_CATALOG_MAX_BYTES,
    })
    .unwrap_err();

    assert!(error.to_string().contains("signature verification failed"));
}

#[test]
fn fast_sync_snapshot_catalog_requires_signature_for_https_sources() {
    let request = SnapshotCatalogLoadRequest {
        source: "https://snapshots.example.com/snapshot-catalog.json".to_string(),
        signature_source: None,
        ed25519_public_key: None,
        max_bytes: FastSyncSnapshotManager::DEFAULT_CATALOG_MAX_BYTES,
    };

    let error = validate_snapshot_catalog_load_request(&request).unwrap_err();

    assert!(error
        .to_string()
        .contains("remote snapshot catalogs require"));
}
