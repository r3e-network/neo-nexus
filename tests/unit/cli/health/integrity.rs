use super::super::*;

#[test]
fn workspace_integrity_cli_reports_healthy_database() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    let repository = Repository::open(&db_path)?;
    repository.create_node(NewNode {
        name: "integrity neo-rs".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: "/usr/local/bin/neo-node".into(),
        args: Vec::new(),
        runtime_version: "v0.8.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: Some(10334),
    })?;
    drop(repository);

    let db_arg = db_path.display().to_string();
    let action = action_from_args(["neo-nexus", "--workspace-integrity", &db_arg])?;
    let CliAction::PrintWithExitCode { text, exit_code } = action else {
        anyhow::bail!("expected integrity action");
    };

    assert_eq!(exit_code, 0);
    assert!(text.contains("workspace-integrity: ok"));
    assert!(text.contains("integrity-check: ok"));
    assert!(text.contains("tables: 14/14"));
    assert!(text.contains("indexes: 5/5"));
    assert!(text.contains("foreign-key-violations: 0"));
    assert!(text.contains("rows: nodes | 1"));
    assert!(text.contains("rows: neo_wallet_profiles | 0"));
    Ok(())
}

#[test]
fn workspace_integrity_json_cli_reports_foreign_key_failure() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    let repository = Repository::open(&db_path)?;
    drop(repository);
    let connection = rusqlite::Connection::open(&db_path)?;
    connection.execute_batch("PRAGMA foreign_keys = OFF;")?;
    connection.execute(
        "INSERT INTO plugin_states (node_id, plugin_id, enabled)
         VALUES ('missing-node', 'RpcServer', 1)",
        [],
    )?;
    drop(connection);

    let db_arg = db_path.display().to_string();
    let action = action_from_args(["neo-nexus", "--workspace-integrity-json", &db_arg])?;
    let CliAction::PrintWithExitCode { text, exit_code } = action else {
        anyhow::bail!("expected integrity JSON action");
    };

    assert_eq!(exit_code, 1);
    let value: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["status"], "failed");
    assert_eq!(value["integrity_check"][0], "ok");
    assert_eq!(value["foreign_key_violations"][0]["table"], "plugin_states");
    assert_eq!(value["foreign_key_violations"][0]["parent_table"], "nodes");
    assert_eq!(value["required_tables"].as_array().map(Vec::len), Some(14));
    assert_eq!(value["required_indexes"].as_array().map(Vec::len), Some(5));
    Ok(())
}
