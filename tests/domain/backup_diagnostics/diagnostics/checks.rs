use super::super::*;

#[test]
fn diagnostics_detect_port_conflicts() {
    let repo = create_repo();
    create_node(&repo, "alpha", NodeType::NeoGo);
    create_node(&repo, "beta", NodeType::NeoGo);
    let nodes = repo.list_nodes().unwrap();
    let diagnostics = evaluate_fleet(&nodes, &BTreeMap::new());

    assert!(diagnostics.critical_count > 0);
    assert!(diagnostics.nodes.iter().any(|node| {
        node.checks
            .iter()
            .any(|check| check.severity == CheckSeverity::Critical && check.title == "Network")
    }));
}

#[test]
fn diagnostics_include_generated_config_validation_for_neo_rs() {
    let temp_dir = tempfile::tempdir().unwrap();
    let binary = temp_dir.path().join("bin").join("neo-node");
    write_fake_executable(&binary);
    let node = NodeConfig {
        id: "neo-rs-1".to_string(),
        name: "neo-rs ready".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: binary,
        args: Vec::new(),
        runtime_version: "v0.8.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: Some(10334),
        status: NodeStatus::Stopped,
        pid: None,
    };

    let diagnostics = evaluate_node(&node, std::slice::from_ref(&node), &[]);

    assert!(diagnostics.checks.iter().any(|check| {
        check.title == "Config storage"
            && check.severity == CheckSeverity::Pass
            && check.detail.contains("rocksdb")
    }));
    assert!(diagnostics.checks.iter().any(|check| {
        check.title == "Config RPC"
            && check.severity == CheckSeverity::Pass
            && check.detail.contains("rpc.port")
    }));
}

#[test]
fn diagnostics_warn_when_runtime_args_bypass_managed_config() {
    let temp_dir = tempfile::tempdir().unwrap();
    let binary = temp_dir.path().join("bin").join("neo-node");
    write_fake_executable(&binary);
    let node = NodeConfig {
        id: "neo-rs-custom-config".to_string(),
        name: "neo-rs custom config".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: binary,
        args: vec!["--config".to_string(), "custom.toml".to_string()],
        runtime_version: "v0.8.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: Some(10334),
        status: NodeStatus::Stopped,
        pid: None,
    };

    let diagnostics = evaluate_node(&node, std::slice::from_ref(&node), &[]);

    assert!(diagnostics.checks.iter().any(|check| {
        check.title == "Managed config"
            && check.severity == CheckSeverity::Warning
            && check.detail.contains("will not inject")
    }));
}

#[test]
fn launch_readiness_warns_when_runtime_args_bypass_managed_config() {
    let temp_dir = tempfile::tempdir().unwrap();
    let binary = temp_dir.path().join("bin").join("neo-node");
    write_fake_executable(&binary);
    let node = NodeConfig {
        id: "candidate".to_string(),
        name: "candidate neo-rs".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: binary,
        args: vec!["--config=custom.toml".to_string()],
        runtime_version: "v0.8.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: None,
        status: NodeStatus::Stopped,
        pid: None,
    };

    let readiness = evaluate_launch_readiness(
        &node,
        std::slice::from_ref(&node),
        &[],
        temp_dir.path().join("generated.toml"),
        temp_dir.path().join("work"),
    );

    assert_eq!(readiness.status_label(), "review");
    assert!(readiness.checks.iter().any(|check| {
        check.title == "Managed config"
            && check.severity == CheckSeverity::Warning
            && check.detail.contains("will not inject")
    }));
    assert!(readiness
        .operator_summary()
        .contains("Managed config: neo-rs runtime args"));
}
