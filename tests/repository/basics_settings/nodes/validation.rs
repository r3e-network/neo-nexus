use super::*;

#[test]
fn rejects_blank_node_names() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();

    let result = repository.create_node(NewNode {
        name: "   ".to_string(),
        node_type: NodeType::NeoCli,
        network: Network::Mainnet,
        binary_path: "/usr/local/bin/neo-cli".into(),
        args: Vec::new(),
        runtime_version: "latest".to_string(),
        storage_engine: StorageEngine::LevelDb,
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: None,
    });

    assert!(result.is_err());
}

#[test]
fn rejects_runtime_incompatible_storage_engines() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();

    let result = repository.create_node(NewNode {
        name: "bad neo-go".to_string(),
        node_type: NodeType::NeoGo,
        network: Network::Testnet,
        binary_path: "/usr/local/bin/neo-go".into(),
        args: Vec::new(),
        runtime_version: "latest".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: None,
    });

    assert!(result.unwrap_err().to_string().contains("does not support"));
}

#[test]
fn rejects_duplicate_ports_within_node_on_create() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();

    let result = repository.create_node(NewNode {
        name: "bad ports".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: "/usr/local/bin/neo-node".into(),
        args: Vec::new(),
        runtime_version: "latest".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 10332,
        p2p_port: 10332,
        ws_port: Some(10333),
    });

    assert!(result
        .unwrap_err()
        .to_string()
        .contains("RPC and P2P ports must be different"));
}

#[test]
fn rejects_duplicate_ports_within_node_on_update() {
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
            ws_port: Some(10334),
        })
        .unwrap();

    let result = repository.update_node(
        &node.id,
        NewNode {
            name: "managed".to_string(),
            node_type: NodeType::NeoCli,
            network: Network::Private,
            binary_path: "/usr/local/bin/neo-cli".into(),
            args: Vec::new(),
            runtime_version: "latest".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 10332,
            p2p_port: 10333,
            ws_port: Some(10332),
        },
    );

    assert!(result
        .unwrap_err()
        .to_string()
        .contains("RPC and WebSocket ports must be different"));
}
