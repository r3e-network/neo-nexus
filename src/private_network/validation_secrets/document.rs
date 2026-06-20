use super::*;

pub(super) fn check_wallet_provisioning_document(
    checks: &mut Vec<LaunchPackValidationCheck>,
    manifest: &DeploymentManifest,
    document: &WalletProvisioningDocument,
) {
    add_check(
        checks,
        "secret-provisioning",
        "wallet-provisioning-json",
        LaunchPackValidationStatus::Pass,
        "wallet provisioning JSON parsed".to_string(),
    );
    add_check(
        checks,
        "secret-provisioning",
        "wallet-provisioning-schema",
        if document.schema_version == WALLET_PROVISIONING_SCHEMA_VERSION {
            LaunchPackValidationStatus::Pass
        } else {
            LaunchPackValidationStatus::Fail
        },
        format!(
            "document schema {}; expected {}",
            document.schema_version, WALLET_PROVISIONING_SCHEMA_VERSION
        ),
    );
    add_check(
        checks,
        "secret-provisioning",
        "wallet-provisioning-policy",
        if document.secret_material_policy == SECRET_PROVISIONING_POLICY
            && document.generated_secret_count == 0
        {
            LaunchPackValidationStatus::Pass
        } else {
            LaunchPackValidationStatus::Fail
        },
        format!(
            "policy {}, generated secrets {}",
            document.secret_material_policy, document.generated_secret_count
        ),
    );

    let expected_entries = wallet_provisioning_entries(manifest);
    add_check(
        checks,
        "secret-provisioning",
        "wallet-provisioning-counts",
        if document.required_wallet_count == expected_entries.len()
            && document.entries.len() == expected_entries.len()
            && document.wallet_reference_count
                == expected_entries
                    .iter()
                    .filter(|entry| entry.wallet_path.is_some())
                    .count()
            && document.missing_wallet_reference_count
                == expected_entries
                    .iter()
                    .filter(|entry| entry.wallet_path.is_none())
                    .count()
        {
            LaunchPackValidationStatus::Pass
        } else {
            LaunchPackValidationStatus::Fail
        },
        format!(
            "document entries {}, expected {}",
            document.entries.len(),
            expected_entries.len()
        ),
    );
    add_check(
        checks,
        "secret-provisioning",
        "wallet-provisioning-entries",
        if document.entries == expected_entries {
            LaunchPackValidationStatus::Pass
        } else {
            LaunchPackValidationStatus::Fail
        },
        "wallet provisioning entries match committee signer references".to_string(),
    );
}
