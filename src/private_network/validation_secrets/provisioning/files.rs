use std::{
    fs,
    path::{Path, PathBuf},
};

use super::super::{
    boundary::check_wallet_provisioning_secret_boundary,
    document::check_wallet_provisioning_document, *,
};

pub(super) fn check_secret_provisioning_files(
    checks: &mut Vec<LaunchPackValidationCheck>,
    root_path: &Path,
    manifest: &DeploymentManifest,
    provisioning: &DeploymentSecretProvisioningManifest,
) {
    let provisioning_path = check_secret_provisioning_file_path(
        checks,
        root_path,
        "wallet-provisioning",
        &provisioning.wallet_provisioning_file,
    );
    let instructions_path = check_secret_provisioning_file_path(
        checks,
        root_path,
        "wallet-instructions",
        &provisioning.wallet_instructions_file,
    );

    if let Some(path) = instructions_path {
        add_file_check(checks, "secret-provisioning", "wallet-instructions", &path);
    }
    if let Some(path) = provisioning_path {
        check_wallet_provisioning_file(checks, manifest, &path);
    }
}

fn check_wallet_provisioning_file(
    checks: &mut Vec<LaunchPackValidationCheck>,
    manifest: &DeploymentManifest,
    path: &Path,
) {
    add_file_check(checks, "secret-provisioning", "wallet-provisioning", path);
    match read_wallet_provisioning_value(path) {
        Ok(value) => {
            check_wallet_provisioning_secret_boundary(checks, &value);
            check_wallet_provisioning_document_value(checks, manifest, value);
        }
        Err(message) => add_wallet_provisioning_json_failure(checks, message),
    }
}

fn check_wallet_provisioning_document_value(
    checks: &mut Vec<LaunchPackValidationCheck>,
    manifest: &DeploymentManifest,
    value: serde_json::Value,
) {
    match serde_json::from_value::<WalletProvisioningDocument>(value) {
        Ok(document) => check_wallet_provisioning_document(checks, manifest, &document),
        Err(error) => add_wallet_provisioning_json_failure(
            checks,
            format!("failed to parse wallet provisioning JSON: {error}"),
        ),
    }
}

fn read_wallet_provisioning_value(path: &Path) -> Result<serde_json::Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("failed to read wallet provisioning JSON: {error}"))?;
    serde_json::from_str::<serde_json::Value>(&text)
        .map_err(|error| format!("failed to parse wallet provisioning JSON: {error}"))
}

fn add_wallet_provisioning_json_failure(
    checks: &mut Vec<LaunchPackValidationCheck>,
    message: String,
) {
    add_check(
        checks,
        "secret-provisioning",
        "wallet-provisioning-json",
        LaunchPackValidationStatus::Fail,
        message,
    );
}

fn check_secret_provisioning_file_path(
    checks: &mut Vec<LaunchPackValidationCheck>,
    root_path: &Path,
    label: &str,
    value: &str,
) -> Option<PathBuf> {
    let Some(path) = safe_launch_pack_child(root_path, value) else {
        add_check(
            checks,
            "secret-provisioning",
            label,
            LaunchPackValidationStatus::Fail,
            format!("path escapes launch pack or is empty: {value}"),
        );
        return None;
    };
    Some(path)
}
