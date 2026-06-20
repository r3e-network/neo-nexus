use super::*;

#[test]
fn updates_node_definition_without_losing_status() {
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
    let updated = repository
        .update_node(
            &node.id,
            NewNode {
                name: "renamed".to_string(),
                node_type: NodeType::NeoGo,
                network: Network::Testnet,
                binary_path: "/opt/neo-go".into(),
                args: vec!["node".into()],
                runtime_version: "v0.110.1".to_string(),
                storage_engine: StorageEngine::LevelDb,
                rpc_port: 20332,
                p2p_port: 20333,
                ws_port: Some(20334),
            },
        )
        .unwrap();

    assert_eq!(updated.name, "renamed");
    assert_eq!(updated.status, NodeStatus::Running);
    assert_eq!(updated.pid, Some(1234));
    assert_eq!(updated.rpc_port, 20332);
    assert_eq!(updated.runtime_version, "v0.110.1");
}

#[test]
fn deletes_node_and_plugin_state() {
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
        .set_plugin_enabled(&node.id, neo_nexus::catalog::PluginId::RpcServer, true)
        .unwrap();
    repository.delete_node(&node.id).unwrap();

    assert!(repository.list_nodes().unwrap().is_empty());
    assert!(repository.list_plugin_states(&node.id).unwrap().is_empty());
}
