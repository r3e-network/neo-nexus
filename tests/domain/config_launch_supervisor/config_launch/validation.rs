use crate::*;

#[test]
fn config_validator_rejects_tampered_neo_rs_toml() {
    let repo = create_repo();
    let node_id = create_node(&repo, "neo-rs", NodeType::NeoRs);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();
    let rendered = ConfigGenerator::render_for_node(&node, &[]).unwrap();
    let tampered = rendered
        .text
        .replace("backend = \"rocksdb\"", "backend = \"leveldb\"")
        .replace("port = 10332", "port = 19999");

    let report = ConfigValidator::validate_text(&node, ConfigFormat::Toml, &tampered);

    assert!(!report.is_success());
    assert_eq!(report.status_label(), "invalid");
    assert!(report
        .checks
        .iter()
        .any(|check| check.title == "Storage backend"
            && check.severity == ConfigValidationSeverity::Critical));
    assert!(report
        .checks
        .iter()
        .any(|check| check.title == "RPC port"
            && check.severity == ConfigValidationSeverity::Critical));
}

#[test]
fn config_validator_rejects_tampered_neo_rs_p2p_operational_limits() {
    let repo = create_repo();
    let node_id = create_node(&repo, "neo-rs", NodeType::NeoRs);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();
    let rendered = ConfigGenerator::render_for_node(&node, &[]).unwrap();
    let tampered = rendered
        .text
        .replace("max_connections = 40", "max_connections = 1")
        .replace("enable_compression = true", "enable_compression = false")
        .replace(
            "broadcast_history_limit = 50000",
            "broadcast_history_limit = 1",
        );

    let report = ConfigValidator::validate_text(&node, ConfigFormat::Toml, &tampered);

    assert!(!report.is_success());
    assert_eq!(report.status_label(), "invalid");
    for title in [
        "P2P max connections",
        "P2P compression",
        "P2P broadcast history",
    ] {
        assert!(
            report.checks.iter().any(|check| check.title == title
                && check.severity == ConfigValidationSeverity::Critical),
            "expected critical finding for {title}"
        );
    }
}

#[test]
fn config_validator_rejects_tampered_neo_rs_consensus_validator_keys() {
    let first_key = committee_public_key("02", 'a');
    let second_key = committee_public_key("03", 'b');
    let profile = RuntimeConfigProfile {
        network_magic: 1_230_405,
        seed_nodes: vec!["127.0.0.1:30333".to_string()],
        validators_count: 2,
        committee_public_keys: vec![first_key.clone(), second_key],
        consensus_enabled: true,
    };
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
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
    let rendered =
        ConfigGenerator::render_for_node_with_profile(&neo_rs, &[], Some(&profile)).unwrap();
    let tampered = rendered
        .text
        .replace(&first_key, &committee_public_key("02", 'f'));

    let report = ConfigValidator::validate_text_with_profile(
        &neo_rs,
        ConfigFormat::Toml,
        &tampered,
        Some(&profile),
    );

    assert!(!report.is_success());
    assert_eq!(report.status_label(), "invalid");
    assert!(report.checks.iter().any(|check| {
        check.title == "Consensus validator keys"
            && check.severity == ConfigValidationSeverity::Critical
    }));
}
