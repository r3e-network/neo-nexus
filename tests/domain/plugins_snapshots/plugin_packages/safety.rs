use super::*;

#[test]
fn plugin_package_manager_rejects_non_neo_cli_nodes() {
    let temp_dir = tempfile::tempdir().unwrap();
    let source = temp_dir.path().join("rpc-server.zip");
    write_zip_snapshot(&source, &[("RpcServer.dll", b"plugin binary")]);
    let (sha256, _) = sha256_file(&source).unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node_id = create_node(&repo, "neo-rs", NodeType::NeoRs);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();

    let error = PluginPackageManager::install(
        &PluginPackageManifest {
            plugin_id: PluginId::RpcServer,
            label: "JSON-RPC API".to_string(),
            source_path: source,
            expected_sha256: sha256,
        },
        &node,
        temp_dir.path().join("nodes").join(&node.id),
    )
    .unwrap_err();

    assert!(error.to_string().contains("neo-cli"));
}

#[test]
fn plugin_package_manager_rejects_checksum_mismatch_before_publish() {
    let temp_dir = tempfile::tempdir().unwrap();
    let source = temp_dir.path().join("rpc-server.zip");
    write_zip_snapshot(&source, &[("RpcServer.dll", b"plugin binary")]);
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node_id = create_node(&repo, "neo-cli", NodeType::NeoCli);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();
    let node_work_dir = temp_dir.path().join("nodes").join(&node.id);

    let error = PluginPackageManager::install(
        &PluginPackageManifest {
            plugin_id: PluginId::RpcServer,
            label: "JSON-RPC API".to_string(),
            source_path: source,
            expected_sha256: "0".repeat(64),
        },
        &node,
        &node_work_dir,
    )
    .unwrap_err();

    assert!(error.to_string().contains("checksum mismatch"));
    assert!(!node_work_dir.join("Plugins/RpcServer").exists());
}

#[test]
fn plugin_package_manager_rejects_unsafe_zip_paths() {
    let temp_dir = tempfile::tempdir().unwrap();
    let source = temp_dir.path().join("evil.zip");
    write_zip_snapshot(&source, &[("../escape.dll", b"nope")]);
    let (sha256, _) = sha256_file(&source).unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node_id = create_node(&repo, "neo-cli", NodeType::NeoCli);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();

    let error = PluginPackageManager::install(
        &PluginPackageManifest {
            plugin_id: PluginId::RpcServer,
            label: "JSON-RPC API".to_string(),
            source_path: source,
            expected_sha256: sha256,
        },
        &node,
        temp_dir.path().join("nodes").join(&node.id),
    )
    .unwrap_err();

    assert!(error.to_string().contains("unsafe"));
    assert!(!temp_dir.path().join("escape.dll").exists());
}
