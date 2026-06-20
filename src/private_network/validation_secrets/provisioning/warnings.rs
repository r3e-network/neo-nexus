use super::super::*;

pub(super) fn warn_missing_wallet_references(
    checks: &mut Vec<LaunchPackValidationCheck>,
    manifest: &DeploymentManifest,
) {
    for signer in &manifest.committee.signers {
        if signer.wallet_path.is_none() {
            add_check(
                checks,
                "wallet-provisioning",
                &signer.label,
                LaunchPackValidationStatus::Warn,
                format!(
                    "no wallet reference; create an encrypted wallet for public key {} and add a signer reference before startup",
                    signer.public_key
                ),
            );
        }
    }
}
