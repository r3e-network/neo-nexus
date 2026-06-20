use super::super::*;

#[test]
fn workspace_readiness_cli_reports_blocking_findings() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    let repository = Repository::open(&db_path)?;
    repository.create_node(NewNode {
        name: "neo-rs blocked".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: "/definitely/missing/neo-node".into(),
        args: Vec::new(),
        runtime_version: "v0.8.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: Some(10334),
    })?;
    drop(repository);

    let db_arg = db_path.display().to_string();
    let action = action_from_args(["neo-nexus", "--workspace-readiness", &db_arg])?;

    assert!(
        matches!(action, CliAction::PrintWithExitCode { text, exit_code: 1 } if text.contains("workspace-readiness: blocked") && text.contains("neo-rs blocked") && text.contains("Binary path"))
    );
    Ok(())
}

#[test]
fn workspace_readiness_json_cli_reports_machine_readable_findings() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    let repository = Repository::open(&db_path)?;
    let created = repository.create_node(NewNode {
        name: "neo-rs blocked".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: "/definitely/missing/neo-node".into(),
        args: Vec::new(),
        runtime_version: "v0.8.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: Some(10334),
    })?;
    drop(repository);

    let db_arg = db_path.display().to_string();
    let action = action_from_args(["neo-nexus", "--workspace-readiness-json", &db_arg])?;

    let (text, exit_code) = match action {
        CliAction::PrintWithExitCode { text, exit_code } => (text, exit_code),
        other => {
            anyhow::bail!("expected JSON readiness action, got {other:?}");
        }
    };
    assert_eq!(exit_code, 1);

    let value: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["status"], "blocked");
    assert_eq!(value["node_count"], 1);
    assert_eq!(value["critical_count"], 1);
    assert_eq!(value["findings"][0]["node_id"], created.id);
    assert_eq!(value["findings"][0]["node_name"], "neo-rs blocked");
    assert_eq!(value["findings"][0]["title"], "Binary path");
    assert_eq!(value["findings"][0]["severity"], "critical");
    Ok(())
}

#[test]
fn workspace_readiness_json_cli_reports_managed_config_bypass_warning() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    let binary_path = temp_dir.path().join("bin").join("neo-node");
    write_fake_executable(&binary_path)?;
    let repository = Repository::open(&db_path)?;
    let created = repository.create_node(NewNode {
        name: "neo-rs custom config".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path,
        args: vec!["--config".to_string(), "custom.toml".to_string()],
        runtime_version: "v0.8.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: Some(10334),
    })?;
    drop(repository);

    let db_arg = db_path.display().to_string();
    let action = action_from_args(["neo-nexus", "--workspace-readiness-json", &db_arg])?;
    let CliAction::PrintWithExitCode { text, exit_code } = action else {
        anyhow::bail!("expected JSON readiness action");
    };

    assert_eq!(exit_code, 0);
    let value: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(value["status"], "review");
    assert_eq!(value["critical_count"], 0);
    assert_eq!(value["warning_count"], 1);
    assert_eq!(value["findings"][0]["node_id"], created.id);
    assert_eq!(value["findings"][0]["title"], "Managed config");
    assert_eq!(value["findings"][0]["severity"], "warning");
    assert!(value["findings"][0]["detail"]
        .as_str()
        .is_some_and(|detail| detail.contains("will not inject")));
    Ok(())
}
