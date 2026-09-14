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
    // The expectation is derived from a workspace this build creates, so the
    // counts move with the schema. What is asserted is that every expected
    // object was found — pinning the number would mean editing this test every
    // time a table is added, which is how the list it replaced drifted.
    let (checked, total) = counted(&text, "tables: ");
    assert_eq!(
        checked, total,
        "a table this build creates was not found: {text}"
    );
    assert!(total >= 24, "only {total} tables checked: {text}");
    let (checked, total) = counted(&text, "indexes: ");
    assert_eq!(
        checked, total,
        "an index this build creates was not found: {text}"
    );
    assert!(total >= 10, "only {total} indexes checked: {text}");
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
    assert!(value["required_tables"]
        .as_array()
        .is_some_and(|t| t.len() >= 24));
    assert!(value["required_indexes"]
        .as_array()
        .is_some_and(|i| i.len() >= 10));

    // The hand-maintained list had already drifted, and these four were the
    // drift. `node_signer_bindings` matters most: its unique index is the
    // constraint that stops two nodes signing with one key, and a workspace
    // missing it passed the integrity check.
    let checked: Vec<&str> = value["required_tables"]
        .as_array()
        .map(|tables| {
            tables
                .iter()
                .filter_map(|table| table["table"].as_str())
                .collect()
        })
        .unwrap_or_default();
    for table in [
        "node_signer_bindings",
        "node_hermes_agents",
        "node_runtime_quarantine",
        "api_tokens",
    ] {
        assert!(
            checked.contains(&table),
            "{table} is not checked; the expected schema has drifted again: {checked:?}"
        );
    }
    Ok(())
}

/// `tables: 24/24` → `(24, 24)`.
fn counted(text: &str, prefix: &str) -> (usize, usize) {
    let line = text
        .lines()
        .find(|line| line.starts_with(prefix))
        .unwrap_or_default();
    let counts = line.trim_start_matches(prefix);
    let mut parts = counts
        .split('/')
        .map(|part| part.trim().parse().unwrap_or(0));
    (parts.next().unwrap_or(0), parts.next().unwrap_or(0))
}
