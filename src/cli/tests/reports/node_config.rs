use super::super::*;

#[test]
fn node_config_export_cli_writes_multi_runtime_configs_and_reports() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    let output_dir = temp_dir.path().join("configs");
    let repository = Repository::open(&db_path)?;
    repository.create_nodes_with_plugins(vec![
        (
            NewNode {
                name: "core rpc".to_string(),
                node_type: NodeType::NeoCli,
                network: Network::Testnet,
                binary_path: "/usr/local/bin/neo-cli".into(),
                args: Vec::new(),
                runtime_version: "v3.8.0".to_string(),
                storage_engine: StorageEngine::RocksDb,
                rpc_port: 10332,
                p2p_port: 10333,
                ws_port: Some(10334),
            },
            vec![
                PluginState {
                    plugin_id: PluginId::RpcServer,
                    enabled: true,
                },
                PluginState {
                    plugin_id: PluginId::StateService,
                    enabled: false,
                },
            ],
        ),
        (
            NewNode {
                name: "go follower".to_string(),
                node_type: NodeType::NeoGo,
                network: Network::Testnet,
                binary_path: "/usr/local/bin/neo-go".into(),
                args: Vec::new(),
                runtime_version: "v0.114.0".to_string(),
                storage_engine: StorageEngine::LevelDb,
                rpc_port: 20332,
                p2p_port: 20333,
                ws_port: Some(20334),
            },
            Vec::new(),
        ),
        (
            NewNode {
                name: "rs validator".to_string(),
                node_type: NodeType::NeoRs,
                network: Network::Testnet,
                binary_path: "/usr/local/bin/neo-node".into(),
                args: Vec::new(),
                runtime_version: "v0.8.0".to_string(),
                storage_engine: StorageEngine::RocksDb,
                rpc_port: 30332,
                p2p_port: 30333,
                ws_port: Some(30334),
            },
            Vec::new(),
        ),
    ])?;
    drop(repository);

    let db_arg = db_path.display().to_string();
    let output_arg = output_dir.display().to_string();
    let action = action_from_args(["neo-nexus", "--export-node-configs", &db_arg, &output_arg])?;

    assert!(
        matches!(action, CliAction::Print(text) if text.contains("node-config-export: ok") && text.contains("nodes: 3") && text.contains("files: 3") && text.contains("report-text:") && text.contains("report-json:") && text.contains("core rpc") && text.contains("rs validator"))
    );

    let report_files = std::fs::read_dir(&output_dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    let json_path = report_files
        .iter()
        .find(|path| path.extension().and_then(|extension| extension.to_str()) == Some("json"))
        .with_context(|| format!("missing JSON report in {}", output_dir.display()))?;
    let value: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(json_path)?)?;
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["status"], "ok");
    assert_eq!(value["database"], db_arg);
    assert_eq!(value["node_count"], 3);
    assert_eq!(value["exported_file_count"], 3);
    assert!(value["total_bytes_written"]
        .as_u64()
        .is_some_and(|bytes| bytes > 0));

    let nodes = value["nodes"]
        .as_array()
        .context("missing exported nodes")?;
    let cli_record = nodes
        .iter()
        .find(|node| node["node_type"] == "neo-cli")
        .context("missing neo-cli config record")?;
    let go_record = nodes
        .iter()
        .find(|node| node["node_type"] == "neo-go")
        .context("missing neo-go config record")?;
    let rs_record = nodes
        .iter()
        .find(|node| node["node_type"] == "neo-rs")
        .context("missing neo-rs config record")?;

    assert_eq!(cli_record["plugin_count"], 2);
    assert_eq!(cli_record["enabled_plugin_count"], 1);
    assert_eq!(cli_record["config_format"], "json");
    assert_eq!(go_record["config_format"], "yaml");
    assert_eq!(rs_record["config_format"], "toml");

    let cli_path = cli_record["path"]
        .as_str()
        .context("missing neo-cli path")?;
    let go_path = go_record["path"].as_str().context("missing neo-go path")?;
    let rs_path = rs_record["path"].as_str().context("missing neo-rs path")?;
    assert!(std::path::Path::new(cli_path).is_file());
    assert!(std::path::Path::new(go_path).is_file());
    assert!(std::path::Path::new(rs_path).is_file());

    let cli_config: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(cli_path)?)?;
    assert_eq!(cli_config["ProtocolConfiguration"]["Network"], 894_710_606);
    assert_eq!(cli_config["Plugins"][0]["Name"], "RpcServer");
    let go_text = std::fs::read_to_string(go_path)?;
    assert!(go_text.contains("ProtocolConfiguration"));
    assert!(go_text.contains("Magic: 894710606"));
    let rs_text = std::fs::read_to_string(rs_path)?;
    assert!(rs_text.contains("[network]"));
    assert!(rs_text.contains("network_magic = 894710606"));

    let json_output_arg = temp_dir.path().join("configs-json").display().to_string();
    let json_action = action_from_args([
        "neo-nexus",
        "--export-node-configs-json",
        &db_arg,
        &json_output_arg,
    ])?;
    let CliAction::Print(json_text) = json_action else {
        anyhow::bail!("expected JSON node config export action");
    };
    let json_value: serde_json::Value = serde_json::from_str(&json_text)?;
    assert_eq!(json_value["status"], "ok");
    assert_eq!(json_value["node_count"], 3);
    assert_eq!(json_value["exported_file_count"], 3);
    Ok(())
}
