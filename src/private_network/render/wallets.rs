use super::{io::*, *};

pub(in crate::private_network) fn render_wallet_provisioning(
    manifest: &DeploymentManifest,
) -> Result<String> {
    let entries = wallet_provisioning_entries(manifest);
    let document = WalletProvisioningDocument {
        schema_version: WALLET_PROVISIONING_SCHEMA_VERSION,
        generated_at_unix: manifest.generated_at_unix,
        runtime: manifest.runtime.clone(),
        template: manifest.template.clone(),
        network_magic: manifest.network_magic,
        secret_material_policy: manifest.secret_provisioning.policy.clone(),
        generated_secret_count: 0,
        required_wallet_count: entries.len(),
        wallet_reference_count: entries
            .iter()
            .filter(|entry| entry.wallet_path.is_some())
            .count(),
        missing_wallet_reference_count: entries
            .iter()
            .filter(|entry| entry.wallet_path.is_none())
            .count(),
        entries,
    };
    serde_json::to_string_pretty(&document).context("failed to render wallet provisioning plan")
}

pub(in crate::private_network) fn wallet_provisioning_entries(
    manifest: &DeploymentManifest,
) -> Vec<WalletProvisioningEntry> {
    manifest
        .committee
        .signers
        .iter()
        .map(|signer| {
            let wallet_path = signer.wallet_path.clone();
            WalletProvisioningEntry {
                label: signer.label.clone(),
                public_key: signer.public_key.clone(),
                recommended_wallet_path: wallet_path.clone().unwrap_or_else(|| {
                    format!("{WALLET_ROOT}/{}.wallet.json", safe_fragment(&signer.label))
                }),
                path_scope: wallet_path_scope(wallet_path.as_deref()).to_string(),
                action: wallet_provisioning_action(wallet_path.as_deref()).to_string(),
                wallet_path,
                signer_endpoint: signer.signer_endpoint.clone(),
                signer_command_template: signer.signer_command_template.clone(),
                signer_command_plan: signer.signer_command_plan.clone(),
            }
        })
        .collect()
}

fn wallet_path_scope(wallet_path: Option<&str>) -> &'static str {
    let Some(wallet_path) = wallet_path else {
        return "missing-reference";
    };
    if is_windows_path(wallet_path) {
        "windows-absolute-or-unc"
    } else if is_posix_absolute_path(wallet_path) {
        "posix-absolute"
    } else {
        "launch-pack-relative"
    }
}

fn wallet_provisioning_action(wallet_path: Option<&str>) -> &'static str {
    if wallet_path.is_some() {
        "create-or-import-encrypted-wallet-at-referenced-path"
    } else {
        "create-encrypted-wallet-and-add-signer-reference"
    }
}

pub(in crate::private_network) fn render_wallet_instructions(
    manifest: &DeploymentManifest,
) -> String {
    let mut text = format!(
        "# NeoNexus Wallet Provisioning\n\nRuntime: `{}`\nTemplate: `{}`\nNetwork magic: `{}`\n\n",
        manifest.runtime, manifest.template, manifest.network_magic
    );
    text.push_str(
        "This directory is intentionally empty except for this README. NeoNexus does not write private keys, wallet passwords, or genesis key material into launch packs.\n\n",
    );
    text.push_str(
        "Use `../wallet-provisioning.json` as the checklist for committee public keys and target wallet paths. Create or import encrypted Neo wallets on the operator host, then re-run `neo-nexus --validate-launch-pack .` from the launch pack root before startup.\n\n",
    );
    text.push_str("Recommended handling:\n\n");
    text.push_str("- Keep wallet files encrypted and owned by the runtime user.\n");
    text.push_str("- Use relative paths under `wallets/` only for disposable local labs.\n");
    text.push_str(
        "- Use absolute platform paths or an external signer for persistent environments.\n",
    );
    text.push_str("- Never commit generated wallet files, private keys, or passwords.\n\n");
    if !manifest.committee.signers.is_empty() {
        text.push_str("Required committee wallets:\n\n");
        for entry in wallet_provisioning_entries(manifest) {
            text.push_str(&format!(
                "- `{}` public_key=`{}` target=`{}` action=`{}`\n",
                entry.label,
                entry.public_key,
                entry
                    .wallet_path
                    .as_deref()
                    .unwrap_or(entry.recommended_wallet_path.as_str()),
                entry.action
            ));
        }
    }
    text
}
