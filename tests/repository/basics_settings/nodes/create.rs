use super::*;

#[test]
fn creates_and_loads_nodes_from_sqlite() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();

    let node = repository
        .create_node(NewNode {
            name: "local testnet".to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Testnet,
            binary_path: "/usr/local/bin/neo-go".into(),
            args: vec!["node".into(), "--config-file".into(), "protocol.yml".into()],
            runtime_version: "v0.110.0".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 10332,
            p2p_port: 10333,
            ws_port: Some(10334),
        })
        .unwrap();

    assert_eq!(node.status, NodeStatus::Stopped);

    let loaded = repository.list_nodes().unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].name, "local testnet");
    assert_eq!(loaded[0].node_type, NodeType::NeoGo);
    assert_eq!(loaded[0].network, Network::Testnet);
    assert_eq!(
        loaded[0].args,
        vec!["node", "--config-file", "protocol.yml"]
    );
}
