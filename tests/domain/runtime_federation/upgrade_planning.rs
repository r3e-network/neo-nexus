use super::*;

#[test]
fn runtime_upgrade_plan_prefers_latest_matching_platform_installation() {
    let platform = RuntimePlatform::current();
    let node = NodeConfig {
        id: "node-a".to_string(),
        name: "neo-rs node".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: PathBuf::from("/old/neo-node"),
        args: Vec::new(),
        runtime_version: "v1.0.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: None,
        status: NodeStatus::Stopped,
        pid: None,
    };
    let older = runtime_install("old", NodeType::NeoRs, "v1.1.0", platform.clone(), 10);
    let newer = runtime_install("new", NodeType::NeoRs, "v1.10.0", platform.clone(), 20);
    let wrong_runtime = runtime_install("go", NodeType::NeoGo, "v9.0.0", platform.clone(), 30);
    let wrong_platform = runtime_install(
        "other-platform",
        NodeType::NeoRs,
        "v9.0.0",
        alternate_platform(&platform),
        40,
    );

    let plan = RuntimePackageManager::plan_node_upgrade(
        &node,
        &[older, newer.clone(), wrong_runtime, wrong_platform],
        &platform,
    )
    .unwrap();

    assert_eq!(plan.package_id, "new");
    assert_eq!(plan.from_version, "v1.0.0");
    assert_eq!(plan.to_version, "v1.10.0");
    assert_eq!(plan.to_binary_path, newer.binary_path);
}

#[test]
fn runtime_catalog_upgrade_plan_prefers_latest_matching_release() {
    let platform = RuntimePlatform::current();
    let node = NodeConfig {
        id: "node-a".to_string(),
        name: "neo-rs node".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: PathBuf::from("/old/neo-node"),
        args: Vec::new(),
        runtime_version: "v1.0.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: None,
        status: NodeStatus::Stopped,
        pid: None,
    };
    let catalog_text = serde_json::json!({
        "schema_version": 1,
        "releases": [
            runtime_release_json("neo-rs-v1-1", NodeType::NeoRs, "v1.1.0", &platform, "a"),
            runtime_release_json("neo-rs-v1-10", NodeType::NeoRs, "v1.10.0", &platform, "b"),
            runtime_release_json("neo-go-v9", NodeType::NeoGo, "v9.0.0", &platform, "c"),
            runtime_release_json(
                "neo-rs-other-platform",
                NodeType::NeoRs,
                "v9.0.0",
                &alternate_platform(&platform),
                "d"
            )
        ]
    })
    .to_string();
    let catalog = RuntimeReleaseCatalog::from_json(&catalog_text).unwrap();

    let plan = RuntimePackageManager::plan_catalog_upgrade(&node, &catalog, &platform).unwrap();

    assert_eq!(plan.node_id, "node-a");
    assert_eq!(plan.from_version, "v1.0.0");
    assert_eq!(plan.to_version, "v1.10.0");
    assert_eq!(plan.release.id, "neo-rs-v1-10");
}

#[test]
fn runtime_catalog_upgrade_plan_skips_current_or_older_release() {
    let platform = RuntimePlatform::current();
    let node = NodeConfig {
        id: "node-a".to_string(),
        name: "neo-rs node".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: PathBuf::from("/current/neo-node"),
        args: Vec::new(),
        runtime_version: "v1.10.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: None,
        status: NodeStatus::Stopped,
        pid: None,
    };
    let catalog_text = serde_json::json!({
        "schema_version": 1,
        "releases": [
            runtime_release_json("neo-rs-v1-2", NodeType::NeoRs, "v1.2.0", &platform, "e"),
            runtime_release_json("neo-rs-v1-10", NodeType::NeoRs, "v1.10.0", &platform, "f")
        ]
    })
    .to_string();
    let catalog = RuntimeReleaseCatalog::from_json(&catalog_text).unwrap();

    let plan = RuntimePackageManager::plan_catalog_upgrade(&node, &catalog, &platform);

    assert!(plan.is_none());
}

#[test]
fn runtime_catalog_fleet_plan_counts_ready_blocked_and_current_nodes() {
    let platform = RuntimePlatform::current();
    let ready = NodeConfig {
        id: "ready".to_string(),
        name: "ready neo-rs".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: PathBuf::from("/old/neo-node"),
        args: Vec::new(),
        runtime_version: "v1.0.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: None,
        status: NodeStatus::Stopped,
        pid: None,
    };
    let running = NodeConfig {
        id: "running".to_string(),
        name: "running neo-rs".to_string(),
        status: NodeStatus::Running,
        pid: Some(42),
        ..ready.clone()
    };
    let current = NodeConfig {
        id: "current".to_string(),
        name: "current neo-rs".to_string(),
        runtime_version: "v1.10.0".to_string(),
        ..ready.clone()
    };
    let unavailable = NodeConfig {
        id: "unavailable".to_string(),
        name: "neo-go unavailable".to_string(),
        node_type: NodeType::NeoGo,
        ..ready.clone()
    };
    let catalog_text = serde_json::json!({
        "schema_version": 1,
        "releases": [
            runtime_release_json("neo-rs-v1-10", NodeType::NeoRs, "v1.10.0", &platform, "a")
        ]
    })
    .to_string();
    let catalog = RuntimeReleaseCatalog::from_json(&catalog_text).unwrap();

    let plan = RuntimePackageManager::plan_catalog_fleet_upgrades(
        &[ready, running, current, unavailable],
        &catalog,
        &platform,
    );

    assert_eq!(plan.stopped_candidates.len(), 1);
    assert_eq!(plan.stopped_candidates[0].node_id, "ready");
    assert_eq!(plan.running_candidates.len(), 1);
    assert_eq!(plan.running_candidates[0].node_id, "running");
    assert_eq!(plan.ready_count(), 2);
    assert_eq!(
        plan.ready_breakdown_label(),
        "2 ready (1 stopped, 1 running)"
    );
    assert_eq!(plan.blocked_active, 0);
    assert_eq!(plan.current_or_unavailable, 2);
}
