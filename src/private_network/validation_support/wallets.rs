use super::super::*;
use super::{
    checks::{add_check, add_file_check},
    paths::{resolve_launch_pack_reference, signer_wallet_path_is_foreign},
};

pub(in crate::private_network) fn check_signer_wallet_reference(
    checks: &mut Vec<LaunchPackValidationCheck>,
    root_path: &Path,
    signer: &DeploymentCommitteeSignerManifest,
    wallet_path: &str,
) {
    if signer_wallet_path_is_foreign(wallet_path) {
        add_check(
            checks,
            "signer-wallet",
            &signer.label,
            LaunchPackValidationStatus::Warn,
            format!("foreign-platform path not checked on this host: {wallet_path}"),
        );
        return;
    }

    let resolved = resolve_launch_pack_reference(root_path, wallet_path);
    add_file_check(checks, "signer-wallet", &signer.label, &resolved);
    check_signer_wallet_format(checks, signer, &resolved);
}

fn check_signer_wallet_format(
    checks: &mut Vec<LaunchPackValidationCheck>,
    signer: &DeploymentCommitteeSignerManifest,
    path: &Path,
) {
    if !path.is_file() {
        return;
    }

    match NeoWalletValidator::validate_path(path) {
        Ok(report) => {
            let format_ready = report.is_success();
            add_check(
                checks,
                "signer-wallet-format",
                &signer.label,
                if format_ready {
                    LaunchPackValidationStatus::Pass
                } else {
                    LaunchPackValidationStatus::Fail
                },
                format!(
                    "NEP-6 wallet {}, {} account(s), {} encrypted account(s), {} failed check(s)",
                    report.status,
                    report.account_count,
                    report.encrypted_account_count,
                    report.failed_count
                ),
            );
            if format_ready {
                check_signer_wallet_committee_key(checks, signer, &report.contract_public_keys);
            }
        }
        Err(error) => add_check(
            checks,
            "signer-wallet-format",
            &signer.label,
            LaunchPackValidationStatus::Fail,
            format!("NEP-6 wallet validation failed: {error}"),
        ),
    }
}

fn check_signer_wallet_committee_key(
    checks: &mut Vec<LaunchPackValidationCheck>,
    signer: &DeploymentCommitteeSignerManifest,
    contract_public_keys: &[String],
) {
    let expected_key = signer
        .public_key
        .trim()
        .trim_start_matches("0x")
        .to_ascii_lowercase();
    let found = contract_public_keys
        .iter()
        .any(|public_key| public_key.eq_ignore_ascii_case(&expected_key));
    add_check(
        checks,
        "signer-wallet-committee-key",
        &signer.label,
        if found {
            LaunchPackValidationStatus::Pass
        } else {
            LaunchPackValidationStatus::Fail
        },
        if found {
            format!("wallet contract includes committee public key {expected_key}")
        } else if contract_public_keys.is_empty() {
            format!(
                "wallet contract exposes no compressed public key for committee public key {expected_key}"
            )
        } else {
            format!(
                "wallet contract public key(s) {} do not include committee public key {expected_key}",
                contract_public_keys.join(", ")
            )
        },
    );
}
