use std::fs;
use tempfile::tempdir;

use super::*;
use crate::types::{Network, NodeConfig, NodeStatus, NodeType, StorageEngine};

fn sample_test_node() -> NodeConfig {
    NodeConfig {
        id: "test-node-01".to_string(),
        name: "test-node-01".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: None,
        binary_path: "neo-node".into(),
        args: Vec::new(),
        runtime_version: "0.1.0".to_string(),
        status: NodeStatus::Stopped,
        pid: None,
    }
}

#[test]
fn config_drift_status_labels() {
    assert_eq!(ConfigDriftStatus::InSync.label(), "in-sync");
    assert_eq!(ConfigDriftStatus::Drifted.label(), "drifted");
    assert_eq!(ConfigDriftStatus::Missing.label(), "missing");
    assert!(ConfigDriftStatus::InSync.is_in_sync());
    assert!(!ConfigDriftStatus::Drifted.is_in_sync());
    assert!(!ConfigDriftStatus::Missing.is_in_sync());
}

#[test]
fn in_sync_configuration_is_detected() {
    let temp_dir = tempdir().unwrap();
    let node = sample_test_node();
    let config_path = temp_dir.path().join("config.toml");

    let rendered = ConfigGenerator::render_for_node(&node, &[]).unwrap();
    fs::write(&config_path, &rendered.text).unwrap();

    let report = ConfigDriftDetector::check(&node, &config_path).unwrap();
    assert_eq!(report.status, ConfigDriftStatus::InSync);
    assert_eq!(report.exit_code(), 0);
    assert!(report.differences.is_empty());
    assert_eq!(report.expected_hash, report.actual_hash.as_deref().unwrap());

    let cli_text = report.to_cli_text();
    assert!(cli_text.contains("config-drift: in-sync"));
    assert!(cli_text.contains("differences: none"));
}

#[test]
fn missing_configuration_is_reported() {
    let temp_dir = tempdir().unwrap();
    let node = sample_test_node();
    let config_path = temp_dir.path().join("non_existent_config.toml");

    let report = ConfigDriftDetector::check(&node, &config_path).unwrap();
    assert_eq!(report.status, ConfigDriftStatus::Missing);
    assert_eq!(report.exit_code(), 1);
    assert!(!report.differences.is_empty());
    assert!(report.actual_hash.is_none());

    let cli_text = report.to_cli_text();
    assert!(cli_text.contains("config-drift: missing"));
}

#[test]
fn drifted_configuration_is_detected_with_differences() {
    let temp_dir = tempdir().unwrap();
    let node = sample_test_node();
    let config_path = temp_dir.path().join("config.toml");

    let tampered_text =
        "# Custom manual edits\n[Protocol]\nNetwork = 9999\n[Storage]\nEngine = 'Memory'\n";
    fs::write(&config_path, tampered_text).unwrap();

    let report = ConfigDriftDetector::check(&node, &config_path).unwrap();
    assert_eq!(report.status, ConfigDriftStatus::Drifted);
    assert_eq!(report.exit_code(), 1);
    assert!(!report.differences.is_empty());

    let cli_text = report.to_cli_text();
    assert!(cli_text.contains("config-drift: drifted"));
    assert!(cli_text.contains("content-hash"));
}

#[test]
fn reconciliation_fixes_drift_and_creates_backup() {
    let temp_dir = tempdir().unwrap();
    let node = sample_test_node();
    let config_path = temp_dir.path().join("config.toml");

    let tampered_text = "# Drifted manually\n[Storage]\nEngine = 'Memory'\n";
    fs::write(&config_path, tampered_text).unwrap();

    let reconcile_report = ConfigReconciler::reconcile(&node, &config_path, true).unwrap();
    assert!(reconcile_report.reconciled);
    assert!(reconcile_report.backup_path.is_some());
    assert_eq!(
        reconcile_report.post_check_status,
        ConfigDriftStatus::InSync
    );

    // Verify backup file exists and contains old text
    let backup_file = reconcile_report.backup_path.clone().unwrap();
    assert!(backup_file.exists());
    let backed_up_content = fs::read_to_string(&backup_file).unwrap();
    assert_eq!(backed_up_content, tampered_text);

    // Verify active file now in-sync
    let re_check = ConfigDriftDetector::check(&node, &config_path).unwrap();
    assert_eq!(re_check.status, ConfigDriftStatus::InSync);

    let cli_text = reconcile_report.to_cli_text();
    assert!(cli_text.contains("config-reconciliation: applied"));
    assert!(cli_text.contains("backup-saved-to"));
}

#[test]
fn reconciliation_skips_when_already_in_sync() {
    let temp_dir = tempdir().unwrap();
    let node = sample_test_node();
    let config_path = temp_dir.path().join("config.toml");

    let rendered = ConfigGenerator::render_for_node(&node, &[]).unwrap();
    fs::write(&config_path, &rendered.text).unwrap();

    let reconcile_report = ConfigReconciler::reconcile(&node, &config_path, true).unwrap();
    assert!(!reconcile_report.reconciled);
    assert!(reconcile_report.backup_path.is_none());
    assert_eq!(
        reconcile_report.post_check_status,
        ConfigDriftStatus::InSync
    );

    let cli_text = reconcile_report.to_cli_text();
    assert!(cli_text.contains("config-reconciliation: already-in-sync"));
}
