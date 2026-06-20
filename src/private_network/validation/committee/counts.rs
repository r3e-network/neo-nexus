use super::*;

pub(super) fn check_committee_summary(
    checks: &mut Vec<LaunchPackValidationCheck>,
    committee: &DeploymentCommitteeManifest,
) {
    add_count_check(
        checks,
        "signer-count",
        committee.signer_count,
        committee.signers.len(),
    );
    add_policy_check(
        checks,
        "secret-policy",
        &committee.secret_material_policy,
        COMMITTEE_SECRET_MATERIAL_POLICY,
    );
    add_policy_check(
        checks,
        "preflight-policy",
        &committee.preflight_policy,
        COMMITTEE_PREFLIGHT_POLICY,
    );
    add_count_check(
        checks,
        "wallet-count",
        committee.wallet_reference_count,
        committee
            .signers
            .iter()
            .filter(|signer| signer.wallet_path.is_some())
            .count(),
    );
    add_count_check(
        checks,
        "endpoint-count",
        committee.endpoint_reference_count,
        committee
            .signers
            .iter()
            .filter(|signer| signer.signer_endpoint.is_some())
            .count(),
    );
    add_count_check(
        checks,
        "sidecar-command-count",
        committee.sidecar_command_count,
        committee
            .signers
            .iter()
            .filter(|signer| signer.signer_command.is_some())
            .count(),
    );
}

fn add_count_check(
    checks: &mut Vec<LaunchPackValidationCheck>,
    label: &'static str,
    expected: usize,
    actual: usize,
) {
    add_check(
        checks,
        "committee",
        label,
        if expected == actual {
            LaunchPackValidationStatus::Pass
        } else {
            LaunchPackValidationStatus::Fail
        },
        format!("manifest says {expected}, contains {actual}"),
    );
}

fn add_policy_check(
    checks: &mut Vec<LaunchPackValidationCheck>,
    label: &'static str,
    actual: &str,
    expected: &str,
) {
    add_check(
        checks,
        "committee",
        label,
        if actual == expected {
            LaunchPackValidationStatus::Pass
        } else {
            LaunchPackValidationStatus::Fail
        },
        actual.to_string(),
    );
}
