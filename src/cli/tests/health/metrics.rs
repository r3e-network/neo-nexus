use super::super::*;

#[test]
fn workspace_metrics_cli_reports_missing_running_processes() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    let repository = Repository::open(&db_path)?;
    let created = repository.create_node(NewNode {
        name: "stale pid neo-rs".to_string(),
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
    repository.update_node_status(&created.id, NodeStatus::Running, Some(999_999))?;
    drop(repository);

    let db_arg = db_path.display().to_string();
    let action = action_from_args(["neo-nexus", "--workspace-metrics", &db_arg])?;

    assert!(
        matches!(action, CliAction::PrintWithExitCode { text, exit_code: 1 }
            if text.contains("workspace-metrics: missing-processes")
                && text.contains("node-processes: 0")
                && text.contains("missing-processes: 1")
                && text.contains("missing: stale pid neo-rs")
                && text.contains("pid 999999"))
    );
    Ok(())
}

#[test]
fn workspace_metrics_json_cli_reports_machine_readable_snapshot() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    let repository = Repository::open(&db_path)?;
    let created = repository.create_node(NewNode {
        name: "json stale pid".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: "/usr/local/bin/neo-node".into(),
        args: Vec::new(),
        runtime_version: "v0.8.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 11332,
        p2p_port: 11333,
        ws_port: Some(11334),
    })?;
    repository.update_node_status(&created.id, NodeStatus::Running, Some(999_998))?;
    drop(repository);

    let db_arg = db_path.display().to_string();
    let action = action_from_args(["neo-nexus", "--workspace-metrics-json", &db_arg])?;

    let (text, exit_code) = match action {
        CliAction::PrintWithExitCode { text, exit_code } => (text, exit_code),
        other => anyhow::bail!("expected workspace metrics JSON action, got {other:?}"),
    };
    assert_eq!(exit_code, 1);
    let value: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["status"], "missing-processes");
    assert_eq!(value["success"], false);
    assert_eq!(
        value["snapshot"]["missing_processes"][0]["node_id"],
        created.id
    );
    assert_eq!(
        value["snapshot"]["missing_processes"][0]["node_name"],
        "json stale pid"
    );
    assert_eq!(value["snapshot"]["missing_processes"][0]["pid"], 999_998);
    assert_eq!(
        value["snapshot"]["node_processes"].as_array().map(Vec::len),
        Some(0)
    );
    assert!(value["snapshot"]["system"]["process_count"]
        .as_u64()
        .is_some_and(|count| count > 0));
    Ok(())
}

#[test]
fn workspace_metrics_prometheus_cli_reports_scrapeable_snapshot() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    let repository = Repository::open(&db_path)?;
    let created = repository.create_node(NewNode {
        name: "prom stale pid".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: "/usr/local/bin/neo-node".into(),
        args: Vec::new(),
        runtime_version: "v0.8.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 12332,
        p2p_port: 12333,
        ws_port: Some(12334),
    })?;
    repository.update_node_status(&created.id, NodeStatus::Running, Some(999_997))?;
    drop(repository);

    let db_arg = db_path.display().to_string();
    let action = action_from_args(["neo-nexus", "--workspace-metrics-prometheus", &db_arg])?;

    let (text, exit_code) = match action {
        CliAction::PrintWithExitCode { text, exit_code } => (text, exit_code),
        other => anyhow::bail!("expected workspace metrics Prometheus action, got {other:?}"),
    };
    assert_eq!(exit_code, 1);
    assert!(text.contains("# HELP neonexus_workspace_status"));
    assert!(text.contains("neonexus_workspace_status 0\n"));
    assert!(text.contains("neonexus_system_processes"));
    assert!(text.contains(&format!(
        "neonexus_node_missing_process{{node_id=\"{}\",node_name=\"prom stale pid\",pid=\"999997\"}} 1\n",
        created.id
    )));
    Ok(())
}
