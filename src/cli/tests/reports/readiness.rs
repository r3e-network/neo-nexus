use super::super::*;

#[test]
fn workspace_readiness_report_cli_writes_text_and_json_evidence() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    let report_dir = temp_dir.path().join("reports");
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
    let report_arg = report_dir.display().to_string();
    let action = action_from_args([
        "neo-nexus",
        "--export-readiness-report",
        &db_arg,
        &report_arg,
    ])?;

    let (text, exit_code) = match action {
        CliAction::PrintWithExitCode { text, exit_code } => (text, exit_code),
        other => {
            anyhow::bail!("expected readiness report action, got {other:?}");
        }
    };
    assert_eq!(exit_code, 1);
    assert!(text.contains("workspace-readiness-report: blocked"));
    assert!(text.contains("report-text:"));
    assert!(text.contains("report-json:"));

    let report_files = std::fs::read_dir(&report_dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    assert_eq!(report_files.len(), 2);
    assert!(report_files
        .iter()
        .any(|path| path.extension().and_then(|extension| extension.to_str()) == Some("txt")));
    let json_path = report_files
        .iter()
        .find(|path| path.extension().and_then(|extension| extension.to_str()) == Some("json"))
        .with_context(|| format!("missing JSON report in {}", report_dir.display()))?;
    let value: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(json_path)?)?;
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["status"], "blocked");
    assert_eq!(value["database"], db_arg);
    assert_eq!(value["nodes"][0]["node_name"], "neo-rs blocked");
    assert_eq!(value["findings"][0]["severity"], "critical");
    assert_eq!(value["findings"][0]["resolution_key"], "runtime-manager");
    assert_eq!(value["findings"][0]["resolution"], "Runtimes");
    assert_eq!(value["findings"][0]["resolution_action"], "Open Runtimes");
    assert_eq!(
        value["findings"][0]["resolution_hint"],
        "Install, verify, or apply node runtime binaries."
    );
    Ok(())
}
