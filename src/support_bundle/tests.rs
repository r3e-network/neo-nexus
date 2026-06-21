use anyhow::Result;

use crate::{
    events::{EventKind, EventSeverity, NewRuntimeEvent},
    logs::LogDiagnosisStatus,
    redaction::{redact_sensitive_args, redact_sensitive_text, REDACTED_VALUE},
    repository::Repository,
    supervisor::log_path_for,
    types::{Network, NewNode, NodeType, StorageEngine},
};

use super::WorkspaceSupportBundleExporter;

mod render_sources;

#[test]
fn support_bundle_redacts_sensitive_argv_shapes() {
    let redacted = redact_sensitive_args(&[
        "--api-key:abc123".to_string(),
        "--mnemonic".to_string(),
        "twelve secret words".to_string(),
        "--authorization".to_string(),
        "Bearer".to_string(),
        "node-access-token".to_string(),
        "--public-rpc".to_string(),
        "http://127.0.0.1:10332".to_string(),
        "--seed=raw-seed".to_string(),
        "--wallet_key".to_string(),
        "wallet-private".to_string(),
    ]);

    assert_eq!(
        redacted,
        [
            "--api-key:<redacted>",
            "--mnemonic",
            REDACTED_VALUE,
            "--authorization",
            REDACTED_VALUE,
            REDACTED_VALUE,
            "--public-rpc",
            "http://127.0.0.1:10332",
            "--seed=<redacted>",
            "--wallet_key",
            REDACTED_VALUE,
        ]
    );
}

#[test]
fn support_bundle_redacts_sensitive_log_excerpt_shapes() {
    let redacted = redact_sensitive_text(
        "Authorization: Bearer node-access-token api_key:abc123 seed=raw-seed webhook=https://hooks.example/token",
    );

    assert!(redacted.contains("Authorization:<redacted>"));
    assert!(redacted.contains("api_key:<redacted>"));
    assert!(redacted.contains("seed=<redacted>"));
    assert!(redacted.contains("webhook=<redacted>"));
    assert!(!redacted.contains("node-access-token"));
    assert!(!redacted.contains("abc123"));
    assert!(!redacted.contains("raw-seed"));
    assert!(!redacted.contains("hooks.example"));
}

#[test]
fn support_bundle_writes_redacted_diagnostics_and_archive() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    let repository = Repository::open(&db_path)?;
    let node = repository.create_node(NewNode {
        name: "neo-rs support".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: "/usr/local/bin/neo-node".into(),
        args: vec![
            "--config".to_string(),
            "custom.toml".to_string(),
            "--wallet-password".to_string(),
            "super-secret".to_string(),
            "--token=abc123".to_string(),
        ],
        runtime_version: "v0.8.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: Some(10334),
    })?;
    let log_path = log_path_for(temp_dir.path().join("logs"), &node);
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(
        &log_path,
        "ERROR bind 127.0.0.1:10332 address already in use token=abc123\n",
    )?;
    repository.record_event_at(
        NewRuntimeEvent {
            node_id: None,
            node_name: None,
            kind: EventKind::BackupExported,
            severity: EventSeverity::Info,
            message: "support test event".to_string(),
        },
        1_800_000_001,
    )?;

    let export = WorkspaceSupportBundleExporter::write_at(
        &repository,
        &db_path,
        temp_dir.path().join("support"),
        "test-version",
        1_800_000_000,
    )?;

    assert_eq!(export.schema_version, 1);
    assert_eq!(export.status, "blocked");
    assert!(export.archive_path.is_file());
    assert!(export.manifest_path.is_file());
    assert!(export
        .manifest
        .files
        .iter()
        .any(|file| file.path == "readiness.json"));
    assert!(export
        .manifest
        .files
        .iter()
        .any(|file| file.path == "nodes.json"));
    assert_eq!(export.manifest.metrics_status, "ok");
    assert_eq!(export.manifest.node_process_count, 0);
    assert_eq!(export.manifest.missing_process_count, 0);
    assert!(export
        .manifest
        .files
        .iter()
        .any(|file| file.path == "metrics.txt"));
    assert!(export
        .manifest
        .files
        .iter()
        .any(|file| file.path == "metrics.json"));
    assert!(export
        .manifest
        .files
        .iter()
        .any(|file| file.path == "metrics.prom"));
    assert_eq!(export.manifest.log_diagnosis_count, 1);
    assert_eq!(export.manifest.log_diagnosis_critical_count, 1);
    assert!(export
        .manifest
        .files
        .iter()
        .any(|file| file.path == "log-diagnosis.json"));
    assert!(export
        .manifest
        .files
        .iter()
        .any(|file| file.path == "log-diagnosis.txt"));

    let nodes = std::fs::read_to_string(export.bundle_dir.join("nodes.json"))?;
    assert!(nodes.contains("\"--wallet-password\""));
    assert!(nodes.contains("\"<redacted>\""));
    assert!(nodes.contains("\"--token=<redacted>\""));
    assert!(!nodes.contains("super-secret"));
    assert!(!nodes.contains("abc123"));

    let privacy = std::fs::read_to_string(export.bundle_dir.join("privacy.txt"))?;
    assert!(privacy.contains("raw workspace database"));
    assert!(privacy.contains("raw runtime logs"));

    let log_diagnosis = std::fs::read_to_string(export.bundle_dir.join("log-diagnosis.json"))?;
    assert!(log_diagnosis.contains(LogDiagnosisStatus::Critical.label()));
    assert!(log_diagnosis.contains("Port binding failure"));
    assert!(log_diagnosis.contains("token=<redacted>"));
    assert!(!log_diagnosis.contains("abc123"));

    let metrics_prom = std::fs::read_to_string(export.bundle_dir.join("metrics.prom"))?;
    assert!(metrics_prom.contains("neonexus_workspace_status 1"));
    assert!(metrics_prom.contains("neonexus_system_processes"));
    Ok(())
}
