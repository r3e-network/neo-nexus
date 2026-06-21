use super::super::*;

#[test]
fn support_bundle_cli_writes_redacted_directory_and_archive() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    let output_dir = temp_dir.path().join("support");
    let binary_path = temp_dir.path().join("bin").join("neo-node");
    write_fake_executable(&binary_path)?;
    let repository = Repository::open(&db_path)?;
    repository.create_node(NewNode {
        name: "neo-rs support".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path,
        args: vec![
            "--config".to_string(),
            "custom.toml".to_string(),
            "--wallet-password".to_string(),
            "super-secret".to_string(),
        ],
        runtime_version: "v0.8.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: Some(10334),
    })?;
    drop(repository);

    let db_arg = db_path.display().to_string();
    let output_arg = output_dir.display().to_string();
    let action = action_from_args(["neo-nexus", "--export-support-bundle", &db_arg, &output_arg])?;

    assert!(
        matches!(action, CliAction::Print(text) if text.contains("support-bundle: review")
            && text.contains("archive-sha256:")
            && text.contains("privacy: diagnostics-only-no-private-keys-passwords-or-webhook-secrets")
            && text.contains("files: 15"))
    );
    let archives = std::fs::read_dir(&output_dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    assert!(archives
        .iter()
        .any(|path| path.extension().and_then(|extension| extension.to_str()) == Some("zip")));
    let bundle_dir = archives
        .iter()
        .find(|path| path.is_dir())
        .with_context(|| {
            format!(
                "missing support bundle directory in {}",
                output_dir.display()
            )
        })?;
    let nodes = std::fs::read_to_string(bundle_dir.join("nodes.json"))?;
    assert!(nodes.contains("\"<redacted>\""));
    assert!(!nodes.contains("super-secret"));
    assert!(bundle_dir.join("manifest.json").is_file());
    assert!(bundle_dir.join("readiness.json").is_file());
    assert!(bundle_dir.join("integrity.json").is_file());
    assert!(bundle_dir.join("metrics.txt").is_file());
    assert!(bundle_dir.join("metrics.json").is_file());
    assert!(bundle_dir.join("metrics.prom").is_file());
    assert!(bundle_dir.join("log-diagnosis.json").is_file());
    assert!(bundle_dir.join("log-diagnosis.txt").is_file());
    Ok(())
}

#[test]
fn support_bundle_json_cli_reports_machine_readable_manifest() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    let output_dir = temp_dir.path().join("support-json");
    let repository = Repository::open(&db_path)?;
    repository.record_event_at(
        NewRuntimeEvent {
            node_id: None,
            node_name: None,
            kind: EventKind::WorkspaceReadinessReportExported,
            severity: EventSeverity::Info,
            message: "support bundle source event".to_string(),
        },
        1_800_000_000,
    )?;
    drop(repository);

    let db_arg = db_path.display().to_string();
    let output_arg = output_dir.display().to_string();
    let action = action_from_args([
        "neo-nexus",
        "--export-support-bundle-json",
        &db_arg,
        &output_arg,
    ])?;
    let CliAction::Print(text) = action else {
        anyhow::bail!("expected support bundle JSON print action");
    };
    let value: serde_json::Value = serde_json::from_str(&text)?;

    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["status"], "ok");
    assert_eq!(
        value["manifest"]["privacy_policy"],
        "diagnostics-only-no-private-keys-passwords-or-webhook-secrets"
    );
    assert_eq!(value["manifest"]["readiness_status"], "ready");
    assert_eq!(value["manifest"]["integrity_status"], "ok");
    assert_eq!(value["manifest"]["matched_event_count"], 1);
    assert_eq!(value["manifest"]["exported_event_count"], 1);
    assert_eq!(value["manifest"]["metrics_status"], "ok");
    assert_eq!(value["manifest"]["node_process_count"], 0);
    assert_eq!(value["manifest"]["missing_process_count"], 0);
    assert_eq!(
        value["manifest"]["files"].as_array().map(Vec::len),
        Some(15)
    );
    assert_eq!(value["manifest"]["log_diagnosis_count"], 0);
    assert!(value["archive_sha256"]
        .as_str()
        .is_some_and(|sha| sha.len() == 64));
    Ok(())
}
