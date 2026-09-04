use super::*;

#[test]
fn persists_rpc_health_records_and_deletes_with_node() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node = repository
        .create_node(NewNode {
            name: "rpc node".to_string(),
            node_type: NodeType::NeoRs,
            network: Network::Testnet,
            binary_path: "/usr/local/bin/neo-node".into(),
            args: Vec::new(),
            runtime_version: "v0.8.0".to_string(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port: 10332,
            p2p_port: 10333,
            ws_port: None,
        })
        .unwrap();

    let first = RpcHealthReport {
        endpoint: "http://127.0.0.1:10332".to_string(),
        status: RpcHealthStatus::Unreachable,
        version: None,
        block_count: None,
        syncing: None,
        network: Default::default(),
        methods: Vec::new(),
    };
    repository
        .record_rpc_health_at(&node, &first, 1_800_000_001)
        .unwrap();
    let second = RpcHealthReport {
        endpoint: "http://127.0.0.1:10332".to_string(),
        status: RpcHealthStatus::Healthy,
        version: Some("neo-rs-test".to_string()),
        block_count: Some(42),
        syncing: Some(false),
        network: neo_nexus::rpc_health::RpcNetworkObservation {
            identity_kind: Some(neo_nexus::rpc_health::RpcIdentityKind::N3NetworkMagic),
            actual_identity: Some(894_710_606),
            expected_identity: Some(894_710_606),
            peer_count: Some(4),
            peers_expected: true,
        },
        methods: Vec::new(),
    };
    let persisted = repository
        .record_rpc_health_at(&node, &second, 1_800_000_002)
        .unwrap();

    assert_eq!(persisted.status, RpcHealthStatus::Healthy);
    assert_eq!(persisted.block_count, Some(42));
    assert_eq!(persisted.syncing, Some(false));
    assert_eq!(persisted.network, second.network);
    assert_eq!(persisted.observed_pid, node.pid);
    assert!(persisted.message.contains("neo-rs-test"));

    let latest = repository.latest_rpc_health(&node.id).unwrap().unwrap();
    assert_eq!(latest.id, persisted.id);
    let history = repository.list_rpc_health(&node.id, 10).unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].status, RpcHealthStatus::Healthy);
    assert_eq!(history[1].status, RpcHealthStatus::Unreachable);
    // Importing an older observation later must not make the fleet/metrics
    // view disagree with latest_rpc_health's timestamp ordering.
    repository
        .record_rpc_health_at(&node, &first, 1_800_000_000)
        .unwrap();
    assert_eq!(
        repository.latest_rpc_health_all_nodes().unwrap()[0].id,
        persisted.id
    );

    repository.delete_node(&node.id).unwrap();

    assert!(repository.latest_rpc_health(&node.id).unwrap().is_none());
    assert!(repository.list_rpc_health(&node.id, 10).unwrap().is_empty());
}

#[test]
fn old_rpc_schema_migrates_without_fabricating_network_or_sync_observations() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("old.db");
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.execute_batch("CREATE TABLE rpc_health_checks (
        id INTEGER PRIMARY KEY AUTOINCREMENT, checked_at_unix INTEGER NOT NULL,
        node_id TEXT NOT NULL, node_name TEXT NOT NULL, endpoint TEXT NOT NULL,
        status TEXT NOT NULL, version TEXT, block_count INTEGER, message TEXT NOT NULL);
        INSERT INTO rpc_health_checks VALUES(1,100,'old-node','old node','http://127.0.0.1:10332','healthy','old',42,'old observation');").unwrap();
    drop(connection);
    let repository = Repository::open(path).unwrap();
    let observation = repository.latest_rpc_health("old-node").unwrap().unwrap();
    assert_eq!(
        observation.network.identity_status(),
        neo_nexus::rpc_health::RpcIdentityStatus::Unknown
    );
    assert_eq!(observation.network.peer_count, None);
    assert_eq!(observation.syncing, None);
    assert_eq!(observation.observed_pid, None);
    assert!(observation.is_fresh(150, 90));
    assert!(!observation.is_fresh(200, 90));
    assert!(!observation.is_fresh(99, 90));
}

#[test]
fn prunes_rpc_health_history_per_node() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let first_node = repository
        .create_node(NewNode {
            name: "rpc primary".to_string(),
            node_type: NodeType::NeoRs,
            network: Network::Testnet,
            binary_path: "/usr/local/bin/neo-node".into(),
            args: Vec::new(),
            runtime_version: "v0.8.0".to_string(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port: 10332,
            p2p_port: 10333,
            ws_port: None,
        })
        .unwrap();
    let second_node = repository
        .create_node(NewNode {
            name: "rpc standby".to_string(),
            node_type: NodeType::NeoRs,
            network: Network::Testnet,
            binary_path: "/usr/local/bin/neo-node".into(),
            args: Vec::new(),
            runtime_version: "v0.8.0".to_string(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port: 10342,
            p2p_port: 10343,
            ws_port: None,
        })
        .unwrap();

    for index in 0..4 {
        let report = RpcHealthReport {
            endpoint: "http://127.0.0.1:10332".to_string(),
            status: if index % 2 == 0 {
                RpcHealthStatus::Healthy
            } else {
                RpcHealthStatus::Degraded
            },
            version: Some(format!("primary-{index}")),
            block_count: Some(100 + index),
            syncing: None,
            network: Default::default(),
            methods: Vec::new(),
        };
        repository
            .record_rpc_health_at(&first_node, &report, 1_800_000_100 + index)
            .unwrap();
    }
    for index in 0..3 {
        let report = RpcHealthReport {
            endpoint: "http://127.0.0.1:10342".to_string(),
            status: RpcHealthStatus::Unreachable,
            version: None,
            block_count: None,
            syncing: None,
            network: Default::default(),
            methods: Vec::new(),
        };
        repository
            .record_rpc_health_at(&second_node, &report, 1_800_000_200 + index)
            .unwrap();
    }

    let removed = repository.prune_rpc_health_keep_recent_per_node(2).unwrap();
    let first_history = repository.list_rpc_health(&first_node.id, 10).unwrap();
    let second_history = repository.list_rpc_health(&second_node.id, 10).unwrap();

    assert_eq!(removed, 3);
    assert_eq!(first_history.len(), 2);
    assert_eq!(first_history[0].checked_at_unix, 1_800_000_103);
    assert_eq!(first_history[1].checked_at_unix, 1_800_000_102);
    assert_eq!(second_history.len(), 2);
    assert_eq!(second_history[0].checked_at_unix, 1_800_000_202);
    assert_eq!(second_history[1].checked_at_unix, 1_800_000_201);
}
