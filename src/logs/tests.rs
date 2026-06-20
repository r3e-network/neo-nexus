use std::path::PathBuf;

use super::*;

#[test]
fn log_diagnosis_detects_port_binding_failures() {
    let snapshot = snapshot_with_lines([
        "INFO booting",
        "ERROR bind 127.0.0.1:10332: address already in use",
    ]);

    let diagnosis = LogReader::diagnose(&snapshot);

    assert_eq!(diagnosis.status, LogDiagnosisStatus::Critical);
    assert_eq!(diagnosis.findings[0].label, "Port binding failure");
    assert!(diagnosis.recommendations[0].contains("Fix Ports"));
}

#[test]
fn log_diagnosis_detects_config_and_database_failures_once() {
    let snapshot = snapshot_with_lines([
        "failed to parse TOML at line 3",
        "invalid config value",
        "rocksdb LOCK resource busy",
    ]);

    let diagnosis = LogReader::diagnose(&snapshot);

    assert_eq!(diagnosis.status, LogDiagnosisStatus::Critical);
    assert_eq!(diagnosis.findings.len(), 2);
    assert!(diagnosis
        .findings
        .iter()
        .any(|finding| finding.label == "Configuration parse failure"));
    assert!(diagnosis
        .findings
        .iter()
        .any(|finding| finding.label == "Database lock"));
}

#[test]
fn log_diagnosis_reports_quiet_logs_without_findings() {
    let snapshot = LogSnapshot {
        path: PathBuf::from("node.log"),
        exists: true,
        bytes: 0,
        truncated: false,
        lines: Vec::new(),
    };

    let diagnosis = LogReader::diagnose(&snapshot);

    assert_eq!(diagnosis.status, LogDiagnosisStatus::Quiet);
    assert!(diagnosis.findings.is_empty());
    assert!(diagnosis.summary.contains("no retained runtime output"));
}

#[test]
fn log_diagnosis_detects_permission_and_runtime_crashes_once() {
    let snapshot = snapshot_with_lines([
        "ERROR permission denied opening /opt/neo-node",
        "fatal: segmentation fault while starting runtime",
        "panic: duplicate crash detail",
    ]);

    let diagnosis = LogReader::diagnose(&snapshot);

    assert_eq!(diagnosis.status, LogDiagnosisStatus::Critical);
    assert_eq!(diagnosis.findings.len(), 2);
    assert!(diagnosis
        .findings
        .iter()
        .any(|finding| finding.label == "Permission failure"));
    assert!(diagnosis
        .findings
        .iter()
        .any(|finding| finding.label == "Runtime crash"));
}

#[test]
fn log_diagnosis_prefers_specific_config_rule_before_generic_failed_rule() {
    let snapshot = snapshot_with_lines([
        "failed to parse TOML at line 3",
        "failed while syncing block",
    ]);

    let diagnosis = LogReader::diagnose(&snapshot);

    assert_eq!(diagnosis.status, LogDiagnosisStatus::Critical);
    assert_eq!(diagnosis.findings[0].label, "Configuration parse failure");
    assert!(diagnosis
        .findings
        .iter()
        .any(|finding| finding.label == "Runtime error"));
}

#[test]
fn log_diagnosis_reports_generic_runtime_warnings() {
    let snapshot = snapshot_with_lines(["INFO ready", "warning: peer retry scheduled"]);

    let diagnosis = LogReader::diagnose(&snapshot);

    assert_eq!(diagnosis.status, LogDiagnosisStatus::Warning);
    assert_eq!(diagnosis.findings[0].label, "Runtime warning");
    assert!(diagnosis.recommendations[0].contains("RPC Health"));
}

fn snapshot_with_lines<const N: usize>(lines: [&str; N]) -> LogSnapshot {
    LogSnapshot {
        path: PathBuf::from("node.log"),
        exists: true,
        bytes: lines.iter().map(|line| line.len() as u64).sum(),
        truncated: false,
        lines: lines.iter().map(ToString::to_string).collect(),
    }
}
