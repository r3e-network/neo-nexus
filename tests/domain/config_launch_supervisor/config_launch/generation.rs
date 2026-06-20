use crate::*;

#[test]
fn config_generator_creates_neo_go_protocol_settings() {
    let repo = create_repo();
    let node_id = create_node(&repo, "neo-go", NodeType::NeoGo);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();

    let rendered = ConfigGenerator::render_for_node(&node, &[]).unwrap();
    let config: serde_yaml::Value = serde_yaml::from_str(&rendered.text).unwrap();

    assert_eq!(rendered.format, ConfigFormat::Yaml);
    assert_eq!(config["ProtocolConfiguration"]["Magic"], 894710606);
    assert_eq!(
        config["ApplicationConfiguration"]["DBConfiguration"]["Type"],
        "leveldb"
    );
    assert_eq!(
        config["ApplicationConfiguration"]["DBConfiguration"]["LevelDBOptions"]
            ["DataDirectoryPath"],
        "data/testnet"
    );
    assert_eq!(config["ApplicationConfiguration"]["RPC"]["Port"], 10332);
    assert_eq!(config["ApplicationConfiguration"]["P2P"]["Port"], 10333);
}

#[test]
fn config_generator_creates_neo_cli_protocol_settings() {
    let repo = create_repo();
    let node_id = create_node(&repo, "neo-cli", NodeType::NeoCli);
    repo.set_plugin_enabled(&node_id, PluginId::RpcServer, true)
        .unwrap();
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();
    let plugins = repo.list_plugin_states(&node.id).unwrap();

    let config = ConfigGenerator::neo_cli(&node, &plugins).unwrap();
    assert_eq!(config["ProtocolConfiguration"]["Network"], 894710606);
    assert_eq!(
        config["ApplicationConfiguration"]["Storage"]["Engine"],
        "RocksDBStore"
    );
    assert_eq!(
        config["ApplicationConfiguration"]["UnlockWallet"]["IsActive"],
        false
    );
    assert_eq!(config["Plugins"][0]["Name"], "RpcServer");
}

#[test]
fn config_generator_creates_neo_rs_toml_settings() {
    let repo = create_repo();
    let node_id = create_node(&repo, "neo-rs", NodeType::NeoRs);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();

    let rendered = ConfigGenerator::render_for_node(&node, &[]).unwrap();

    assert_eq!(rendered.format, ConfigFormat::Toml);
    assert!(rendered.text.contains("[network]"));
    assert!(rendered.text.contains("network_type = \"testnet\""));
    assert!(rendered.text.contains("backend = \"rocksdb\""));
    assert!(rendered.text.contains("port = 10332"));

    let report = ConfigValidator::validate_rendered(&node, &rendered);
    assert!(report.is_success());
    assert_eq!(report.status_label(), "ready");
    assert!(report
        .checks
        .iter()
        .any(|check| check.title == "Storage backend"
            && check.severity == ConfigValidationSeverity::Pass));
}

#[test]
fn config_generator_applies_private_network_runtime_profile() {
    let profile = RuntimeConfigProfile {
        network_magic: 1_230_404,
        seed_nodes: vec!["127.0.0.1:30333".to_string(), "127.0.0.1:30343".to_string()],
        validators_count: 4,
        committee_public_keys: vec![
            committee_public_key("02", 'a'),
            committee_public_key("03", 'b'),
            committee_public_key("02", 'c'),
            committee_public_key("03", 'd'),
        ],
        consensus_enabled: true,
    };
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let neo_cli = repo
        .create_node(NewNode {
            name: "private cli".to_string(),
            node_type: NodeType::NeoCli,
            network: Network::Private,
            binary_path: PathBuf::from("/usr/local/bin/neo-cli"),
            args: Vec::new(),
            runtime_version: "v3.8.0".to_string(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port: 30332,
            p2p_port: 30333,
            ws_port: None,
        })
        .unwrap();
    let neo_go = repo
        .create_node(NewNode {
            name: "private go".to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Private,
            binary_path: PathBuf::from("/usr/local/bin/neo-go"),
            args: Vec::new(),
            runtime_version: "v0.110.0".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 30342,
            p2p_port: 30343,
            ws_port: None,
        })
        .unwrap();
    let neo_rs = repo
        .create_node(NewNode {
            name: "private rs".to_string(),
            node_type: NodeType::NeoRs,
            network: Network::Private,
            binary_path: PathBuf::from("/usr/local/bin/neo-node"),
            args: Vec::new(),
            runtime_version: "v0.8.0".to_string(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port: 30352,
            p2p_port: 30353,
            ws_port: None,
        })
        .unwrap();

    let cli_config = ConfigGenerator::for_node_with_profile(&neo_cli, &[], Some(&profile)).unwrap();
    assert_eq!(cli_config["ProtocolConfiguration"]["Network"], 1_230_404);
    assert_eq!(
        cli_config["ProtocolConfiguration"]["SeedList"][0],
        "127.0.0.1:30333"
    );
    assert_eq!(cli_config["ProtocolConfiguration"]["ValidatorsCount"], 4);
    assert_eq!(
        cli_config["ProtocolConfiguration"]["StandbyCommittee"][0],
        committee_public_key("02", 'a')
    );

    let go_text = ConfigGenerator::render_for_node_with_profile(&neo_go, &[], Some(&profile))
        .unwrap()
        .text;
    let go_config: serde_yaml::Value = serde_yaml::from_str(&go_text).unwrap();
    assert_eq!(go_config["ProtocolConfiguration"]["Magic"], 1_230_404);
    assert_eq!(go_config["ProtocolConfiguration"]["ValidatorsCount"], 4);
    assert_eq!(
        go_config["ProtocolConfiguration"]["SeedList"][1],
        "127.0.0.1:30343"
    );
    assert_eq!(
        go_config["ProtocolConfiguration"]["StandbyCommittee"][1],
        committee_public_key("03", 'b')
    );

    let rs_rendered =
        ConfigGenerator::render_for_node_with_profile(&neo_rs, &[], Some(&profile)).unwrap();
    let rs_config: toml::Value = toml::from_str(&rs_rendered.text).unwrap();
    assert_eq!(
        rs_config["network"]["network_magic"].as_integer(),
        Some(1_230_404)
    );
    assert_eq!(
        rs_config["p2p"]["seed_nodes"][0].as_str(),
        Some("127.0.0.1:30333")
    );
    assert_eq!(
        rs_config["consensus"]["validators"][2].as_str(),
        Some(committee_public_key("02", 'c').as_str())
    );
    assert_eq!(rs_config["consensus"]["enabled"].as_bool(), Some(true));

    let rs_report =
        ConfigValidator::validate_rendered_with_profile(&neo_rs, &rs_rendered, Some(&profile));
    assert!(rs_report.is_success());
    assert_eq!(rs_report.status_label(), "ready");
}
