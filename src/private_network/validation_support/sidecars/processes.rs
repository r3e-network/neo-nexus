use super::super::checks::add_check;
use super::{super::super::*, binaries::signer_sidecar_process_binary_path};

pub(in crate::private_network) fn check_signer_sidecar_process_spec(
    checks: &mut Vec<LaunchPackValidationCheck>,
    root_path: &Path,
    signer: &DeploymentCommitteeSignerManifest,
    plan: &SignerCommandPlan,
) {
    match deployment_signer_sidecar_process(root_path, signer, plan) {
        Ok(sidecar) => add_check(
            checks,
            "signer-sidecar-process-spec",
            &signer.label,
            LaunchPackValidationStatus::Pass,
            format!(
                "{} -> {} (log {})",
                sidecar.process.id,
                sidecar.process.binary_path.display(),
                sidecar.log_path.display()
            ),
        ),
        Err(error) => add_check(
            checks,
            "signer-sidecar-process-spec",
            &signer.label,
            LaunchPackValidationStatus::Fail,
            error.to_string(),
        ),
    }
}

pub(in crate::private_network) fn committee_sidecar_process(
    launch_pack_root: &Path,
    label: &str,
    public_key: &str,
    wallet_path: Option<PathBuf>,
    signer_endpoint: Option<String>,
    plan: &SignerCommandPlan,
) -> Result<CommitteeSidecarProcess> {
    validate_signer_command_plan(plan)?;
    let binary_path = signer_sidecar_process_binary_path(launch_pack_root, &plan.binary);
    let sidecar_dir = signer_sidecar_work_dir(launch_pack_root, label);
    let log_path = sidecar_dir.join(format!("{label}.supervisor.log"));
    let display_command = sh_command_tokens(&binary_path.display().to_string(), &plan.arguments);

    Ok(CommitteeSidecarProcess {
        signer_label: label.to_string(),
        public_key: public_key.to_string(),
        wallet_path,
        signer_endpoint,
        log_path,
        process: ManagedProcessSpec {
            id: format!("signer:{label}"),
            kind: ManagedProcessKind::Sidecar,
            label: label.to_string(),
            binary_path,
            args: plan.arguments.clone(),
            working_dir: launch_pack_root.to_path_buf(),
            display_command,
        },
    })
}

pub(in crate::private_network) fn deployment_sidecar_processes(
    root_path: &Path,
    committee: &DeploymentCommitteeManifest,
) -> Result<Vec<CommitteeSidecarProcess>> {
    committee
        .signers
        .iter()
        .filter_map(|signer| {
            signer
                .signer_command_plan
                .as_ref()
                .map(|plan| (signer, plan))
        })
        .map(|(signer, plan)| deployment_signer_sidecar_process(root_path, signer, plan))
        .collect()
}

fn signer_sidecar_work_dir(root_path: &Path, label: &str) -> PathBuf {
    root_path.join(SIGNER_SIDECAR_ROOT).join(label)
}

fn deployment_signer_sidecar_process(
    root_path: &Path,
    signer: &DeploymentCommitteeSignerManifest,
    plan: &SignerCommandPlan,
) -> Result<CommitteeSidecarProcess> {
    committee_sidecar_process(
        root_path,
        &signer.label,
        &signer.public_key,
        signer.wallet_path.as_deref().map(PathBuf::from),
        signer.signer_endpoint.clone(),
        plan,
    )
}
