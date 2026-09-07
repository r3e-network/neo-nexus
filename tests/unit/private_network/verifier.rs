//! Tests for private network launch pack verification error handling

use std::path::PathBuf;

use crate::private_network::PrivateNetworkLaunchPackVerifier;

fn missing_root() -> PathBuf {
    PathBuf::from("this-launch-pack-directory-does-not-exist-12345")
}

#[test]
fn validate_reports_missing_manifest_file() {
    let error = PrivateNetworkLaunchPackVerifier::validate(missing_root())
        .expect_err("missing manifest must fail validation");
    assert!(
        error
            .to_string()
            .contains("failed to read launch pack manifest"),
        "unexpected error message: {error}"
    );
}

#[test]
fn sidecar_report_fails_when_manifest_is_missing() {
    let error = PrivateNetworkLaunchPackVerifier::sidecar_report(missing_root())
        .expect_err("missing manifest must fail the sidecar report");
    assert!(
        error
            .to_string()
            .contains("failed to read launch pack manifest"),
        "unexpected error message: {error}"
    );
}

#[test]
fn sidecar_processes_fails_when_manifest_is_missing() {
    let result = PrivateNetworkLaunchPackVerifier::sidecar_processes(missing_root());
    assert!(result.is_err(), "missing manifest must fail");
}
