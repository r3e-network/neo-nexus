use super::super::*;

#[test]
fn launch_readiness_blocks_active_port_conflicts_before_start() {
    let temp_dir = tempfile::tempdir().unwrap();
    let binary = temp_dir.path().join("bin").join("neo-node");
    write_fake_executable(&binary);
    let active = NodeConfig {
        id: "active".to_string(),
        name: "active neo-rs".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: binary.clone(),
        args: Vec::new(),
        runtime_version: "v0.8.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: Some(10334),
        status: NodeStatus::Running,
        pid: Some(42),
    };
    let candidate = NodeConfig {
        id: "candidate".to_string(),
        name: "candidate neo-rs".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: binary,
        args: Vec::new(),
        runtime_version: "v0.8.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 10332,
        p2p_port: 20333,
        ws_port: None,
        status: NodeStatus::Stopped,
        pid: None,
    };
    let nodes = vec![active, candidate.clone()];

    let readiness = evaluate_launch_readiness(
        &candidate,
        &nodes,
        &[],
        temp_dir.path().join("candidate.toml"),
        temp_dir.path().join("candidate-work"),
    );

    assert_eq!(readiness.status_label(), "blocked");
    assert!(readiness.checks.iter().any(|check| {
        check.title == "Launch ports"
            && check.severity == CheckSeverity::Critical
            && check.detail.contains("RPC port 10332")
    }));
}

#[test]
fn launch_readiness_blocks_localhost_port_listeners_before_start() {
    let temp_dir = tempfile::tempdir().unwrap();
    let binary = temp_dir.path().join("bin").join("neo-node");
    write_fake_executable(&binary);
    let node = NodeConfig {
        id: "candidate".to_string(),
        name: "candidate neo-rs".to_string(),
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

    let readiness = evaluate_launch_readiness_with_port_probe(
        &node,
        std::slice::from_ref(&node),
        &[],
        temp_dir.path().join("generated.toml"),
        temp_dir.path().join("work"),
        |port| port != 10332,
    );

    assert_eq!(readiness.status_label(), "blocked");
    assert!(readiness.checks.iter().any(|check| {
        check.title == "Launch ports"
            && check.severity == CheckSeverity::Critical
            && check.detail.contains("RPC port 10332")
            && check.detail.contains("127.0.0.1")
            && check.detail.contains("Fix Ports")
    }));
}

#[test]
fn restart_readiness_allows_current_node_localhost_ports_before_restart() {
    let temp_dir = tempfile::tempdir().unwrap();
    let binary = temp_dir.path().join("bin").join("neo-node");
    write_fake_executable(&binary);
    let node = NodeConfig {
        id: "running".to_string(),
        name: "running neo-rs".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: binary,
        args: Vec::new(),
        runtime_version: "v0.8.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: Some(10334),
        status: NodeStatus::Running,
        pid: Some(4242),
    };

    let readiness = evaluate_restart_readiness_with_port_probe(
        &node,
        std::slice::from_ref(&node),
        &[],
        temp_dir.path().join("generated.toml"),
        temp_dir.path().join("work"),
        |_| false,
    );

    assert_eq!(readiness.status_label(), "ready");
    assert!(readiness.checks.iter().any(|check| {
        check.title == "Restart lifecycle"
            && check.severity == CheckSeverity::Pass
            && check.detail.contains("eligible for managed restart")
    }));
    assert!(!readiness.checks.iter().any(|check| {
        check.title == "Launch ports"
            && check.severity == CheckSeverity::Critical
            && check.detail.contains("127.0.0.1")
    }));
}

#[test]
fn restart_readiness_blocks_other_active_node_port_conflicts() {
    let temp_dir = tempfile::tempdir().unwrap();
    let binary = temp_dir.path().join("bin").join("neo-node");
    write_fake_executable(&binary);
    let active = NodeConfig {
        id: "active".to_string(),
        name: "active neo-rs".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: binary.clone(),
        args: Vec::new(),
        runtime_version: "v0.8.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 10332,
        p2p_port: 20333,
        ws_port: None,
        status: NodeStatus::Running,
        pid: Some(42),
    };
    let restarting = NodeConfig {
        id: "restarting".to_string(),
        name: "restarting neo-rs".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: binary,
        args: Vec::new(),
        runtime_version: "v0.8.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: Some(10334),
        status: NodeStatus::Running,
        pid: Some(43),
    };
    let nodes = vec![active, restarting.clone()];

    let readiness = evaluate_restart_readiness_with_port_probe(
        &restarting,
        &nodes,
        &[],
        temp_dir.path().join("generated.toml"),
        temp_dir.path().join("work"),
        |_| true,
    );

    assert_eq!(readiness.status_label(), "blocked");
    assert!(readiness.checks.iter().any(|check| {
        check.title == "Launch ports"
            && check.severity == CheckSeverity::Critical
            && check.detail.contains("RPC port 10332")
            && check.detail.contains("active neo-rs")
    }));
}
