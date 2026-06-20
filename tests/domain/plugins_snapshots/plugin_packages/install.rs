use super::*;

#[test]
fn plugin_package_manager_installs_verified_neo_cli_zip_package() {
    let temp_dir = tempfile::tempdir().unwrap();
    let source = temp_dir.path().join("rpc-server.zip");
    write_zip_snapshot(
        &source,
        &[
            ("RpcServer.dll", b"plugin binary"),
            ("config.json", br#"{"port":10332}"#),
        ],
    );
    let (sha256, package_bytes) = sha256_file(&source).unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node_id = create_node(&repo, "neo-cli", NodeType::NeoCli);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();

    let installation = PluginPackageManager::install(
        &PluginPackageManifest {
            plugin_id: PluginId::RpcServer,
            label: "JSON-RPC API".to_string(),
            source_path: source.clone(),
            expected_sha256: sha256.clone(),
        },
        &node,
        temp_dir.path().join("nodes").join(&node.id),
    )
    .unwrap();

    assert_eq!(installation.plugin_id, PluginId::RpcServer);
    assert_eq!(installation.sha256, sha256);
    assert_eq!(installation.package_bytes, package_bytes);
    assert_eq!(installation.installed_files, 2);
    assert_eq!(
        std::fs::read_to_string(installation.installed_path.join("RpcServer.dll")).unwrap(),
        "plugin binary"
    );
    let manifest = std::fs::read_to_string(&installation.manifest_path).unwrap();
    assert!(manifest.contains("\"plugin_id\": \"RpcServer\""));
    assert!(manifest.contains(&node.id));
}

#[test]
fn plugin_package_manager_replaces_existing_package_safely() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node_id = create_node(&repo, "neo-cli", NodeType::NeoCli);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();
    let node_work_dir = temp_dir.path().join("nodes").join(&node.id);
    let source_v1 = temp_dir.path().join("rpc-server-v1.zip");
    let source_v2 = temp_dir.path().join("rpc-server-v2.zip");
    write_zip_snapshot(&source_v1, &[("RpcServer.dll", b"version one")]);
    write_zip_snapshot(&source_v2, &[("RpcServer.dll", b"version two")]);
    let (sha_v1, _) = sha256_file(&source_v1).unwrap();
    let (sha_v2, _) = sha256_file(&source_v2).unwrap();

    PluginPackageManager::install(
        &PluginPackageManifest {
            plugin_id: PluginId::RpcServer,
            label: "JSON-RPC API".to_string(),
            source_path: source_v1,
            expected_sha256: sha_v1,
        },
        &node,
        &node_work_dir,
    )
    .unwrap();
    let installation = PluginPackageManager::install(
        &PluginPackageManifest {
            plugin_id: PluginId::RpcServer,
            label: "JSON-RPC API".to_string(),
            source_path: source_v2,
            expected_sha256: sha_v2,
        },
        &node,
        &node_work_dir,
    )
    .unwrap();

    assert_eq!(
        std::fs::read_to_string(installation.installed_path.join("RpcServer.dll")).unwrap(),
        "version two"
    );
    assert!(!node_work_dir
        .join("Plugins/.neonexus/replace-backups")
        .read_dir()
        .map(|mut entries| entries.next().is_some())
        .unwrap_or(false));
}
