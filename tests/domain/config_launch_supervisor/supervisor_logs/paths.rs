use crate::*;

#[test]
fn supervisor_log_path_is_stable_per_node() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node_id = repo
        .create_node(NewNode {
            name: "neo-rs logger".to_string(),
            node_type: NodeType::NeoRs,
            network: Network::Testnet,
            binary_path: PathBuf::from("/usr/local/bin/neo-node"),
            args: Vec::new(),
            runtime_version: "v0.8.0".to_string(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port: 10332,
            p2p_port: 10333,
            ws_port: None,
        })
        .unwrap()
        .id;
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();

    let log_path = log_path_for(temp_dir.path().join("logs"), &node);

    assert_eq!(
        log_path,
        temp_dir
            .path()
            .join("logs")
            .join(format!("neo-rs-{}.log", node.id))
    );
}
