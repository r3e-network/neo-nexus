//! Tests for launch-pack validation and sidecar report rendering

use std::path::PathBuf;

use crate::private_network::{
    LaunchPackValidationCheck, LaunchPackValidationStatus, PrivateNetworkLaunchPackSidecarReport,
    PrivateNetworkLaunchPackValidation,
};

fn validation(checks: Vec<LaunchPackValidationCheck>) -> PrivateNetworkLaunchPackValidation {
    let failed = checks
        .iter()
        .filter(|check| check.status == LaunchPackValidationStatus::Fail)
        .count();
    let warnings = checks
        .iter()
        .filter(|check| check.status == LaunchPackValidationStatus::Warn)
        .count();
    let passed = checks
        .iter()
        .filter(|check| check.status == LaunchPackValidationStatus::Pass)
        .count();
    PrivateNetworkLaunchPackValidation {
        root_path: PathBuf::from("/srv/launch-pack"),
        manifest_path: PathBuf::from("/srv/launch-pack/manifest.json"),
        schema_version: 1,
        node_count: 4,
        signer_count: 2,
        passed_count: passed,
        warning_count: warnings,
        failed_count: failed,
        checks,
    }
}

fn check(status: LaunchPackValidationStatus) -> LaunchPackValidationCheck {
    LaunchPackValidationCheck {
        category: "schema".to_string(),
        label: "version".to_string(),
        status,
        message: "ok".to_string(),
    }
}

#[test]
fn validation_is_success_only_when_no_failures() {
    assert!(validation(vec![check(LaunchPackValidationStatus::Pass)]).is_success());
    assert!(validation(vec![check(LaunchPackValidationStatus::Warn)]).is_success());
    assert!(!validation(vec![check(LaunchPackValidationStatus::Fail)]).is_success());
}

#[test]
fn validation_cli_text_reports_status_and_check_lines() {
    let text = validation(vec![check(LaunchPackValidationStatus::Fail)]).to_cli_text();
    assert!(text.contains("launch-pack: failed"));
    assert!(text.contains("schema: 1"));
    assert!(text.contains("nodes: 4"));
    assert!(text.contains("FAIL [schema] version: ok"));
}

#[test]
fn validation_status_serializes_to_kebab_case() {
    let json = serde_json::to_string(&LaunchPackValidationStatus::Pass).unwrap();
    assert_eq!(json, "\"pass\"");
    let json = serde_json::to_string(&LaunchPackValidationStatus::Warn).unwrap();
    assert_eq!(json, "\"warn\"");
}

#[test]
fn sidecar_report_status_and_text_reflect_emptiness() {
    let empty = PrivateNetworkLaunchPackSidecarReport {
        root_path: PathBuf::from("/srv/launch-pack"),
        manifest_path: PathBuf::from("/srv/launch-pack/manifest.json"),
        sidecar_count: 0,
        sidecars: Vec::new(),
    };
    assert_eq!(empty.status_label(), "empty");
    let text = empty.to_cli_text();
    assert!(text.contains("launch-pack-sidecars: empty"));
    assert!(text.contains("sidecar: none"));
}
