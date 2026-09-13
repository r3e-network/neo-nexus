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

#[test]
fn plugin_package_manager_rejects_active_nodes_before_write() {
    let temp_dir = tempfile::tempdir().unwrap();
    let source = temp_dir.path().join("rpc-server.zip");
    write_zip_snapshot(&source, &[("RpcServer.dll", b"plugin binary")]);
    let (sha256, _) = sha256_file(&source).unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node_id = create_node(&repo, "neo-cli", NodeType::NeoCli);
    let mut node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();
    // Activate the node so that install eligibility fails early
    repo.update_node_status(&node.id, NodeStatus::Running, Some(1234))
        .unwrap();
    // Reload with updated status
    node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();

    let error = PluginPackageManager::install(
        &PluginPackageManifest {
            plugin_id: PluginId::RpcServer,
            label: "JSON-RPC API".to_string(),
            source_path: source.clone(),
            expected_sha256: sha256,
        },
        &node,
        temp_dir.path().join("nodes").join(&node.id),
    )
    .unwrap_err();

    assert!(error.to_string().contains("stop and settle"));
    assert!(!temp_dir
        .path()
        .join("nodes")
        .join(&node.id)
        .join("Plugins")
        .exists());
}

#[test]
fn plugin_package_manager_rejects_nodes_with_pid_before_write() {
    let temp_dir = tempfile::tempdir().unwrap();
    let source = temp_dir.path().join("rpc-server.zip");
    write_zip_snapshot(&source, &[("RpcServer.dll", b"plugin binary")]);
    let (sha256, _) = sha256_file(&source).unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node_id = create_node(&repo, "neo-cli", NodeType::NeoCli);
    let mut node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();
    // Record a PID so that install eligibility fails early
    repo.update_node_status(&node.id, NodeStatus::Running, Some(12345))
        .unwrap();
    // Reload with updated status
    node = repo
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

    assert!(error.to_string().contains("stop and settle"));
    assert!(!temp_dir
        .path()
        .join("nodes")
        .join(&node.id)
        .join("Plugins")
        .exists());
}

#[test]
fn neo_go_package_installer_fails_without_side_effects() {
    let temp_dir = tempfile::tempdir().unwrap();
    let source = temp_dir.path().join("rpc-server.zip");
    write_zip_snapshot(&source, &[("RpcServer.dll", b"plugin binary")]);
    let (sha256, _) = sha256_file(&source).unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node_id = create_node(&repo, "neo-go", NodeType::NeoGo);
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
            expected_sha256: sha256,
        },
        &node,
        &node_work_dir,
    )
    .unwrap_err();

    assert!(error.to_string().contains("C# DLL"));
    assert!(error.to_string().contains("neo-cli"));
    assert!(!node_work_dir.exists());
}

#[test]
fn catalog_placeholder_install_module_returns_error_no_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    let go_mod = temp_dir.path().join("go.mod");
    std::fs::write(&go_mod, "module example.com/node\n").unwrap();
    let working_dir = temp_dir.path();

    let result = neo_nexus::catalog::neo_go_modules::install_module("echo", working_dir);
    assert!(result.is_err());

    let error = result.unwrap_err().to_string();
    assert!(error.contains("automated source integration and rebuilding are not implemented"));
    // Verify no files were changed or created
    assert_eq!(
        std::fs::read_to_string(go_mod).unwrap(),
        "module example.com/node\n"
    );
}

#[test]
fn catalog_placeholder_toggle_module_returns_error_no_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_yaml = temp_dir.path().join("config.yaml");
    std::fs::write(&config_yaml, "network:\n  magic: mainnet\n").unwrap();
    let working_dir = temp_dir.path();

    let result = neo_nexus::catalog::neo_go_modules::toggle_module("stateroot", true, working_dir);
    assert!(result.is_err());

    let error = result.unwrap_err().to_string();
    assert!(error.contains("generic module configuration is not implemented"));
    // Verify no files were changed
    assert!(std::fs::read_to_string(config_yaml)
        .unwrap()
        .starts_with("network:"));
}

#[test]
fn catalog_feature_toggle_returns_error_no_changes() {
    use neo_nexus::config::GenerationContext;

    let temp_dir = tempfile::tempdir().unwrap();
    let cargo_toml = temp_dir.path().join("Cargo.toml");
    std::fs::write(
        &cargo_toml,
        "[package]\nname = \"neo-rs-test\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    let ctx = GenerationContext::default();

    let result =
        neo_nexus::catalog::neo_rs_features::toggle_feature("rocksdb-backend", false, &ctx);
    assert!(result.is_err());

    let error = result.unwrap_err().to_string();
    assert!(error.contains("automated Cargo feature management is not implemented"));
    // Verify no changes to Cargo.toml
    let content = std::fs::read_to_string(cargo_toml).unwrap();
    assert!(content.contains("neo-rs-test"));
    assert!(!content.contains("rocksdb"));
}
