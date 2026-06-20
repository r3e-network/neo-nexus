use super::*;

pub(super) fn check_committee_signer(
    checks: &mut Vec<LaunchPackValidationCheck>,
    root_path: &Path,
    signer: &DeploymentCommitteeSignerManifest,
    seen_keys: &mut BTreeMap<String, String>,
) {
    check_signer_key(checks, signer, seen_keys);
    if let Some(wallet_path) = &signer.wallet_path {
        check_signer_wallet_reference(checks, root_path, signer, wallet_path);
    }
    if let Some(endpoint) = &signer.signer_endpoint {
        check_signer_endpoint(checks, signer, endpoint);
    }
    if let Some(command) = &signer.signer_command {
        check_signer_sidecar(checks, root_path, signer, command);
    }
}

fn check_signer_key(
    checks: &mut Vec<LaunchPackValidationCheck>,
    signer: &DeploymentCommitteeSignerManifest,
    seen_keys: &mut BTreeMap<String, String>,
) {
    add_check(
        checks,
        "committee",
        format!("{} key", signer.label),
        match normalize_public_key(&signer.public_key) {
            Ok(key) => {
                if seen_keys.insert(key, signer.label.clone()).is_some() {
                    LaunchPackValidationStatus::Fail
                } else {
                    LaunchPackValidationStatus::Pass
                }
            }
            Err(_) => LaunchPackValidationStatus::Fail,
        },
        signer.public_key.clone(),
    );
}

fn check_signer_endpoint(
    checks: &mut Vec<LaunchPackValidationCheck>,
    signer: &DeploymentCommitteeSignerManifest,
    endpoint: &str,
) {
    add_check(
        checks,
        "signer-endpoint",
        &signer.label,
        if validate_signer_endpoint(endpoint).is_ok() {
            LaunchPackValidationStatus::Pass
        } else {
            LaunchPackValidationStatus::Fail
        },
        endpoint.to_string(),
    );
}

fn check_signer_sidecar(
    checks: &mut Vec<LaunchPackValidationCheck>,
    root_path: &Path,
    signer: &DeploymentCommitteeSignerManifest,
    command: &str,
) {
    add_check(
        checks,
        "signer-sidecar",
        &signer.label,
        if validate_signer_command(command).is_ok() {
            LaunchPackValidationStatus::Pass
        } else {
            LaunchPackValidationStatus::Fail
        },
        command.to_string(),
    );
    add_check(
        checks,
        "signer-sidecar-plan",
        &signer.label,
        signer_command_plan_status(signer, command),
        signer
            .signer_command_plan
            .as_ref()
            .map(|plan| format!("{} {} args", plan.execution_policy, plan.arguments.len()))
            .unwrap_or_else(|| "missing argv execution plan".to_string()),
    );
    if let Some(plan) = &signer.signer_command_plan {
        check_signer_sidecar_binary(checks, root_path, signer, plan);
        check_signer_sidecar_process_spec(checks, root_path, signer, plan);
    }
    if let Some(template) = &signer.signer_command_template {
        check_signer_sidecar_template(checks, signer, template);
    }
    if signer.signer_endpoint.is_none() {
        add_check(
            checks,
            "signer-sidecar",
            format!("{} endpoint", signer.label),
            LaunchPackValidationStatus::Warn,
            "sidecar command has no endpoint reference for post-start health checks".to_string(),
        );
    }
}

fn check_signer_sidecar_template(
    checks: &mut Vec<LaunchPackValidationCheck>,
    signer: &DeploymentCommitteeSignerManifest,
    template: &str,
) {
    add_check(
        checks,
        "signer-sidecar-template",
        &signer.label,
        if validate_signer_command_template(template).is_ok() {
            LaunchPackValidationStatus::Pass
        } else {
            LaunchPackValidationStatus::Fail
        },
        template.to_string(),
    );
}
