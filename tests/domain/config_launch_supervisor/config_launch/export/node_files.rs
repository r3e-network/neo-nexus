use crate::*;

#[test]
fn config_exporter_writes_safe_json_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node_id = repo
        .create_node(NewNode {
            name: "Neo CLI / validator #1".to_string(),
            node_type: NodeType::NeoCli,
            network: Network::Private,
            binary_path: PathBuf::from("/usr/local/bin/neo-cli"),
            args: Vec::new(),
            runtime_version: "v3.8.0".to_string(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port: 20332,
            p2p_port: 20333,
            ws_port: None,
        })
        .unwrap()
        .id;
    repo.set_plugin_enabled(&node_id, PluginId::RpcServer, true)
        .unwrap();
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();
    let plugins = repo.list_plugin_states(&node.id).unwrap();

    let export =
        ConfigExporter::write_node_config(temp_dir.path().join("configs"), &node, &plugins)
            .unwrap();
    let text = std::fs::read_to_string(&export.path).unwrap();

    assert!(export
        .path
        .ends_with("neo-cli-validator-1-neo-cli-config.json"));
    assert_eq!(export.bytes_written, text.len());
    assert!(text.contains("\"Network\": 1230000"));
    assert!(text.contains("\"RpcServer\""));
}

#[test]
fn config_exporter_writes_neo_go_yaml_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node_id = repo
        .create_node(NewNode {
            name: "Neo Go RPC".to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Testnet,
            binary_path: PathBuf::from("/usr/local/bin/neo-go"),
            args: Vec::new(),
            runtime_version: "v0.110.0".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 30332,
            p2p_port: 30333,
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

    let export =
        ConfigExporter::write_node_config(temp_dir.path().join("configs"), &node, &[]).unwrap();
    let text = std::fs::read_to_string(&export.path).unwrap();
    let parsed: serde_yaml::Value = serde_yaml::from_str(&text).unwrap();

    assert!(export.path.ends_with("neo-go-rpc-neo-go-config.yml"));
    assert_eq!(export.bytes_written, text.len());
    assert_eq!(parsed["ProtocolConfiguration"]["Magic"], 894710606);
    assert_eq!(parsed["ApplicationConfiguration"]["RPC"]["Port"], 30332);
}

#[test]
fn config_exporter_writes_neo_rs_toml_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node_id = repo
        .create_node(NewNode {
            name: "Neo RS validator".to_string(),
            node_type: NodeType::NeoRs,
            network: Network::Mainnet,
            binary_path: PathBuf::from("/usr/local/bin/neo-node"),
            args: vec!["--config".to_string(), "mainnet.toml".to_string()],
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

    let export =
        ConfigExporter::write_node_config(temp_dir.path().join("configs"), &node, &[]).unwrap();
    let text = std::fs::read_to_string(&export.path).unwrap();

    assert!(export.path.ends_with("neo-rs-validator-neo-rs-config.toml"));
    assert_eq!(export.bytes_written, text.len());
    assert!(text.contains("network_type = \"mainnet\""));
    assert!(text.contains("network_magic = 860833102"));
    assert!(text.contains("data_dir = \"./data/mainnet\""));
}
