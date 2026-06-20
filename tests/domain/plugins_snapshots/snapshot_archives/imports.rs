use super::*;

#[test]
fn fast_sync_snapshot_manager_imports_neo_rs_tar_gzip_snapshot_package() {
    let temp_dir = tempfile::tempdir().unwrap();
    let source = temp_dir.path().join("neo-rs-testnet.tar.gz");
    write_tar_gzip_snapshot(
        &source,
        &[
            ("chain/CURRENT", b"manifest pointer"),
            ("chain/000001.sst", b"rocksdb state bytes"),
        ],
    );
    let (sha256, package_bytes) = sha256_file(&source).unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node_id = create_node(&repo, "neo-rs", NodeType::NeoRs);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();
    let cache = FastSyncSnapshotManager::cache(
        &local_snapshot("neo-rs-targz", NodeType::NeoRs, source.clone(), &sha256),
        temp_dir.path().join("cache"),
    )
    .unwrap();
    let snapshot = FastSyncSnapshot {
        cached_path: Some(cache.path),
        verified_sha256: Some(sha256.clone()),
        verified_at_unix: Some(cache.cached_at_unix),
        bytes: Some(package_bytes),
        download_file_name: Some("neo-rs-testnet.tar.gz".to_string()),
        ..local_snapshot("neo-rs-targz", NodeType::NeoRs, source, &sha256)
    };
    let node_data_dir = temp_dir
        .path()
        .join("nodes")
        .join(&node.id)
        .join("data/testnet");

    let application =
        FastSyncSnapshotManager::apply_to_node(&snapshot, &node, &node_data_dir).unwrap();

    assert_eq!(application.import_mode, SnapshotImportMode::TarGzipArchive);
    assert_eq!(application.imported_files, 2);
    assert_eq!(
        application.expanded_bytes,
        "manifest pointer".len() as u64 + "rocksdb state bytes".len() as u64
    );
    assert_eq!(application.snapshot_path, node_data_dir);
    assert_eq!(
        std::fs::read_to_string(node_data_dir.join("chain/CURRENT")).unwrap(),
        "manifest pointer"
    );
    assert_eq!(
        std::fs::read_to_string(node_data_dir.join("chain/000001.sst")).unwrap(),
        "rocksdb state bytes"
    );
    let manifest = std::fs::read_to_string(&application.manifest_path).unwrap();
    assert!(manifest.contains("\"import_mode\": \"tar-gzip\""));
    assert!(
        manifest.contains("\"runtime_import_profile\": \"neo-rs [storage].data_dir RocksDB data\"")
    );
}

#[test]
fn fast_sync_snapshot_manager_imports_zip_snapshot_package() {
    let temp_dir = tempfile::tempdir().unwrap();
    let source = temp_dir.path().join("neo-cli-mainnet.zip");
    write_zip_snapshot(
        &source,
        &[
            ("Chain/main.dat", b"leveldb bytes"),
            ("Chain/MANIFEST-000001", b"manifest"),
        ],
    );
    let (sha256, _) = sha256_file(&source).unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node_id = create_node(&repo, "neo-cli", NodeType::NeoCli);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();
    let cache = FastSyncSnapshotManager::cache(
        &local_snapshot("neo-cli-zip", NodeType::NeoCli, source.clone(), &sha256),
        temp_dir.path().join("cache"),
    )
    .unwrap();
    let snapshot = FastSyncSnapshot {
        cached_path: Some(cache.path),
        verified_sha256: Some(sha256.clone()),
        verified_at_unix: Some(cache.cached_at_unix),
        download_file_name: Some("neo-cli-mainnet.zip".to_string()),
        ..local_snapshot("neo-cli-zip", NodeType::NeoCli, source, &sha256)
    };
    let node_data_dir = temp_dir
        .path()
        .join("nodes")
        .join(&node.id)
        .join("data/testnet");

    let application =
        FastSyncSnapshotManager::apply_to_node(&snapshot, &node, &node_data_dir).unwrap();

    assert_eq!(application.import_mode, SnapshotImportMode::ZipArchive);
    assert_eq!(application.imported_files, 2);
    assert_eq!(
        std::fs::read_to_string(node_data_dir.join("Chain/main.dat")).unwrap(),
        "leveldb bytes"
    );
}
