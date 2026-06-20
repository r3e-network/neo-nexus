use super::super::*;

pub(super) fn check_secret_provisioning_metadata(
    checks: &mut Vec<LaunchPackValidationCheck>,
    manifest: &DeploymentManifest,
    provisioning: &DeploymentSecretProvisioningManifest,
) {
    check_schema_version(checks, provisioning);
    check_policy(checks, provisioning);
    check_generated_secret_count(checks, provisioning);
    check_wallet_reference_counts(checks, manifest, provisioning);
}

fn check_schema_version(
    checks: &mut Vec<LaunchPackValidationCheck>,
    provisioning: &DeploymentSecretProvisioningManifest,
) {
    add_check(
        checks,
        "secret-provisioning",
        "schema",
        if provisioning.schema_version == WALLET_PROVISIONING_SCHEMA_VERSION {
            LaunchPackValidationStatus::Pass
        } else {
            LaunchPackValidationStatus::Fail
        },
        format!(
            "wallet provisioning schema {}; expected {}",
            provisioning.schema_version, WALLET_PROVISIONING_SCHEMA_VERSION
        ),
    );
}

fn check_policy(
    checks: &mut Vec<LaunchPackValidationCheck>,
    provisioning: &DeploymentSecretProvisioningManifest,
) {
    add_check(
        checks,
        "secret-provisioning",
        "policy",
        if provisioning.policy == SECRET_PROVISIONING_POLICY {
            LaunchPackValidationStatus::Pass
        } else {
            LaunchPackValidationStatus::Fail
        },
        provisioning.policy.clone(),
    );
}

fn check_generated_secret_count(
    checks: &mut Vec<LaunchPackValidationCheck>,
    provisioning: &DeploymentSecretProvisioningManifest,
) {
    add_check(
        checks,
        "secret-provisioning",
        "generated-secrets",
        if provisioning.generated_secret_count == 0 {
            LaunchPackValidationStatus::Pass
        } else {
            LaunchPackValidationStatus::Fail
        },
        format!("{} generated secrets", provisioning.generated_secret_count),
    );
}

fn check_wallet_reference_counts(
    checks: &mut Vec<LaunchPackValidationCheck>,
    manifest: &DeploymentManifest,
    provisioning: &DeploymentSecretProvisioningManifest,
) {
    let signer_count = manifest.committee.signers.len();
    let wallet_reference_count = manifest
        .committee
        .signers
        .iter()
        .filter(|signer| signer.wallet_path.is_some())
        .count();
    let missing_wallet_reference_count = signer_count.saturating_sub(wallet_reference_count);

    add_check(
        checks,
        "secret-provisioning",
        "required-wallet-count",
        if provisioning.required_wallet_count == signer_count {
            LaunchPackValidationStatus::Pass
        } else {
            LaunchPackValidationStatus::Fail
        },
        format!(
            "manifest says {}, committee requires {}",
            provisioning.required_wallet_count, signer_count
        ),
    );
    add_check(
        checks,
        "secret-provisioning",
        "wallet-reference-count",
        if provisioning.wallet_reference_count == wallet_reference_count
            && provisioning.missing_wallet_reference_count == missing_wallet_reference_count
        {
            LaunchPackValidationStatus::Pass
        } else {
            LaunchPackValidationStatus::Fail
        },
        format!(
            "references {} / missing {}, committee references {} / missing {}",
            provisioning.wallet_reference_count,
            provisioning.missing_wallet_reference_count,
            wallet_reference_count,
            missing_wallet_reference_count
        ),
    );
}
