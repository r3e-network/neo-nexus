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

#[test]
fn plugin_upgrade_requires_config_review_and_keeps_local_keys() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let id = create_node(&repo, "neo-cli", NodeType::NeoCli);
    let mut node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == id)
        .unwrap();
    node.runtime_version = "3.8.0".into();
    let work = temp_dir.path().join("nodes").join(&id);
    let source = temp_dir.path().join("plugin.zip");
    let install = |version: &str| {
        let (sha, _) = sha256_file(&source).unwrap();
        PluginPackageManager::install_with_release(
            &PluginPackageManifest {
                plugin_id: PluginId::RpcServer,
                label: "RPC".into(),
                source_path: source.clone(),
                expected_sha256: sha,
            },
            &node,
            &work,
            Some(&neo_nexus::plugins::PluginReleaseMetadata {
                version: version.into(),
                compatible_runtime_versions: vec![node.runtime_version.clone()],
            }),
        )
    };
    write_zip_snapshot(
        &source,
        &[("RpcServer.dll", b"one"), ("config.json", b"{\"port\":1}")],
    );
    let original = install("1.0").unwrap();
    let config = original.installed_path.join("config.json");
    std::fs::write(&config, b"{\"port\":9,\"password\":\"local\"}").unwrap();
    std::fs::write(original.installed_path.join("wallet.key"), b"secret-key").unwrap();
    write_zip_snapshot(
        &source,
        &[("RpcServer.dll", b"two"), ("config.json", b"{\"port\":2}")],
    );
    assert!(install("2.0")
        .unwrap_err()
        .to_string()
        .contains("configuration conflicts"));
    assert_eq!(
        std::fs::read(original.installed_path.join("RpcServer.dll")).unwrap(),
        b"one"
    );
    let conflict = neo_nexus::config::config_conflict(&config)
        .unwrap()
        .unwrap();
    assert_eq!(conflict.to_version, "2.0");
    neo_nexus::config::resolve_config_conflict(&config, &conflict.token, true).unwrap();
    let upgraded = install("2.0").unwrap();
    assert_eq!(
        std::fs::read(upgraded.installed_path.join("RpcServer.dll")).unwrap(),
        b"two"
    );
    assert_eq!(
        std::fs::read(&config).unwrap(),
        b"{\"port\":9,\"password\":\"local\"}"
    );
    assert_eq!(
        std::fs::read(upgraded.installed_path.join("wallet.key")).unwrap(),
        b"secret-key"
    );
    assert_eq!(
        neo_nexus::plugins::installed_plugin_release(&upgraded.manifest_path)
            .unwrap()
            .unwrap()
            .version,
        "2.0"
    );
}

#[test]
fn versioned_plugin_rejects_incompatible_runtime_before_install() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let id = create_node(&repo, "neo-cli", NodeType::NeoCli);
    let mut node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == id)
        .unwrap();
    node.runtime_version = "3.8.0".into();
    let source = temp_dir.path().join("plugin.zip");
    write_zip_snapshot(&source, &[("RpcServer.dll", b"one")]);
    let (sha, _) = sha256_file(&source).unwrap();
    let work = temp_dir.path().join("node");
    let error = PluginPackageManager::install_with_release(
        &PluginPackageManifest {
            plugin_id: PluginId::RpcServer,
            label: "RPC".into(),
            source_path: source,
            expected_sha256: sha,
        },
        &node,
        &work,
        Some(&neo_nexus::plugins::PluginReleaseMetadata {
            version: "2.0".into(),
            compatible_runtime_versions: vec!["999.0.0".into()],
        }),
    )
    .unwrap_err();
    assert!(error.to_string().contains("not declared compatible"));
    assert!(!work.exists());
}

#[test]
fn disabled_plugin_is_outside_loader_tree_and_stays_disabled_when_upgraded() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let id = create_node(&repo, "neo-cli", NodeType::NeoCli);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == id)
        .unwrap();
    let work = temp_dir.path().join("node");
    let source = temp_dir.path().join("plugin.zip");
    let install = || {
        let (sha, _) = sha256_file(&source).unwrap();
        PluginPackageManager::install(
            &PluginPackageManifest {
                plugin_id: PluginId::RpcServer,
                label: "RPC".into(),
                source_path: source.clone(),
                expected_sha256: sha,
            },
            &node,
            &work,
        )
        .unwrap()
    };
    write_zip_snapshot(&source, &[("RpcServer.dll", b"one")]);
    let installation = install();
    repo.upsert_plugin_installation(&installation).unwrap();
    repo.set_plugin_enabled(&id, PluginId::RpcServer, false)
        .unwrap();
    PluginPackageManager::synchronize_enabled(&repo, &work, &node).unwrap();
    assert!(!work.join("Plugins/RpcServer").exists());
    let disabled = work.join(".neonexus-disabled-plugins/RpcServer");
    assert_eq!(
        repo.list_plugin_installations(&id).unwrap()[0].installed_path,
        disabled
    );
    write_zip_snapshot(&source, &[("RpcServer.dll", b"two")]);
    assert_eq!(install().installed_path, disabled);
    assert!(!work.join("Plugins/RpcServer").exists());
    repo.set_plugin_enabled(&id, PluginId::RpcServer, true)
        .unwrap();
    PluginPackageManager::synchronize_enabled(&repo, &work, &node).unwrap();
    assert_eq!(
        std::fs::read(work.join("Plugins/RpcServer/RpcServer.dll")).unwrap(),
        b"two"
    );
    assert!(!disabled.exists());
}
