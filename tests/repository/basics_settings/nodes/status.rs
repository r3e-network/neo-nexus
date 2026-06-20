use super::*;

#[test]
fn updates_node_status() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node = repository
        .create_node(NewNode {
            name: "managed".to_string(),
            node_type: NodeType::NeoCli,
            network: Network::Private,
            binary_path: "/usr/local/bin/neo-cli".into(),
            args: Vec::new(),
            runtime_version: "latest".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 10332,
            p2p_port: 10333,
            ws_port: None,
        })
        .unwrap();

    repository
        .update_node_status(&node.id, NodeStatus::Running, Some(1234))
        .unwrap();

    let loaded = repository.list_nodes().unwrap();
    assert_eq!(loaded[0].status, NodeStatus::Running);
    assert_eq!(loaded[0].pid, Some(1234));
}

#[test]
fn clears_transient_runtime_state_from_previous_sessions() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let running = repository
        .create_node(NewNode {
            name: "running".to_string(),
            node_type: NodeType::NeoCli,
            network: Network::Private,
            binary_path: "/usr/local/bin/neo-cli".into(),
            args: Vec::new(),
            runtime_version: "latest".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 10332,
            p2p_port: 10333,
            ws_port: None,
        })
        .unwrap();
    let starting = repository
        .create_node(NewNode {
            name: "starting".to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Testnet,
            binary_path: "/usr/local/bin/neo-go".into(),
            args: Vec::new(),
            runtime_version: "latest".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 20332,
            p2p_port: 20333,
            ws_port: None,
        })
        .unwrap();

    repository
        .update_node_status(&running.id, NodeStatus::Running, Some(111))
        .unwrap();
    repository
        .update_node_status(&starting.id, NodeStatus::Starting, Some(222))
        .unwrap();

    let changed = repository.clear_transient_runtime_state().unwrap();

    let loaded = repository.list_nodes().unwrap();
    assert_eq!(changed, 2);
    assert!(loaded
        .iter()
        .all(|node| node.status == NodeStatus::Stopped && node.pid.is_none()));
}
