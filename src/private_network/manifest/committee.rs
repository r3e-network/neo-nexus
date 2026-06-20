use super::super::*;

pub(in crate::private_network) fn committee_manifest(
    committee: Option<&CommitteeRoster>,
) -> DeploymentCommitteeManifest {
    let signers = committee_signers(committee);
    DeploymentCommitteeManifest {
        signer_count: signers.len(),
        wallet_reference_count: count_wallet_references(&signers),
        endpoint_reference_count: count_endpoint_references(&signers),
        sidecar_command_count: count_sidecar_commands(&signers),
        public_keys: signers
            .iter()
            .map(|signer| signer.public_key.clone())
            .collect(),
        secret_material_policy: COMMITTEE_SECRET_MATERIAL_POLICY.to_string(),
        preflight_policy: COMMITTEE_PREFLIGHT_POLICY.to_string(),
        signers,
    }
}

pub(in crate::private_network) fn secret_provisioning_manifest(
    committee: Option<&CommitteeRoster>,
) -> DeploymentSecretProvisioningManifest {
    let signer_count = committee.map_or(0, |committee| committee.signers.len());
    let wallet_reference_count = committee.map_or(0, |committee| {
        committee
            .signers
            .iter()
            .filter(|signer| signer.wallet_path.is_some())
            .count()
    });
    DeploymentSecretProvisioningManifest {
        schema_version: WALLET_PROVISIONING_SCHEMA_VERSION,
        policy: SECRET_PROVISIONING_POLICY.to_string(),
        wallet_provisioning_file: WALLET_PROVISIONING_FILE.to_string(),
        wallet_instructions_file: WALLET_INSTRUCTIONS_FILE.to_string(),
        recommended_wallet_root: WALLET_ROOT.to_string(),
        required_wallet_count: signer_count,
        wallet_reference_count,
        missing_wallet_reference_count: signer_count.saturating_sub(wallet_reference_count),
        generated_secret_count: 0,
    }
}

fn committee_signers(
    committee: Option<&CommitteeRoster>,
) -> Vec<DeploymentCommitteeSignerManifest> {
    committee
        .map(|committee| {
            committee
                .signers
                .iter()
                .map(|signer| DeploymentCommitteeSignerManifest {
                    label: signer.label.clone(),
                    public_key: signer.public_key.clone(),
                    wallet_path: signer
                        .wallet_path
                        .as_ref()
                        .map(|path| path.display().to_string()),
                    signer_endpoint: signer.signer_endpoint.clone(),
                    signer_command_template: signer.signer_command_template.clone(),
                    signer_command: signer.signer_command.clone(),
                    signer_command_plan: signer.signer_command_plan.clone(),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn count_wallet_references(signers: &[DeploymentCommitteeSignerManifest]) -> usize {
    signers
        .iter()
        .filter(|signer| signer.wallet_path.is_some())
        .count()
}

fn count_endpoint_references(signers: &[DeploymentCommitteeSignerManifest]) -> usize {
    signers
        .iter()
        .filter(|signer| signer.signer_endpoint.is_some())
        .count()
}

fn count_sidecar_commands(signers: &[DeploymentCommitteeSignerManifest]) -> usize {
    signers
        .iter()
        .filter(|signer| signer.signer_command.is_some())
        .count()
}
