use super::*;

pub(in crate::private_network) fn render_start_order(manifest: &DeploymentManifest) -> String {
    let mut text = format!(
        "NeoNexus private network launch pack\nruntime: {}\ntemplate: {}\nmagic: {}\nvalidators: {}\n\n",
        manifest.runtime, manifest.template, manifest.network_magic, manifest.validators_count
    );
    for node in &manifest.nodes {
        text.push_str(&format!(
            "{}. {} [{}] {}\n   config: {}\n   config_sha256: {}\n   workdir: {}\n\n",
            node.start_order,
            node.name,
            node.role,
            node.command,
            node.config_path,
            node.config_sha256,
            node.working_dir
        ));
    }
    text
}

pub(in crate::private_network) fn render_runbook(manifest: &DeploymentManifest) -> String {
    let mut text = format!(
        "# NeoNexus Private Network Runbook\n\nRuntime: `{}`\nTemplate: `{}`\nNetwork magic: `{}`\nValidators: `{}`\n\n",
        manifest.runtime, manifest.template, manifest.network_magic, manifest.validators_count
    );
    text.push_str(
        "## Operator Flow\n\n1. Place required node binaries, config files, work directories, and signer wallet references on this host.\n2. Run `neo-nexus --validate-launch-pack .` from this directory and inspect `validation-report.txt` / `validation-report.json`.\n3. Run the platform preflight script.\n4. Run the platform start script.\n5. Run the platform health script until all nodes are ready.\n6. Run the platform stop script when shutting down the lab.\n\n",
    );
    text.push_str("## Platform Commands\n\n");
    text.push_str(&format!(
        "- Unix/macOS preflight: `./{}`\n- Unix/macOS start: `./{}`\n- Unix/macOS health: `./{}`\n- Unix/macOS stop: `./{}`\n",
        manifest.scripts.preflight_unix,
        manifest.scripts.start_unix,
        manifest.scripts.health_unix,
        manifest.scripts.stop_unix
    ));
    text.push_str(&format!(
        "- Windows preflight: `powershell -ExecutionPolicy Bypass -File .\\{}`\n- Windows start: `powershell -ExecutionPolicy Bypass -File .\\{}`\n- Windows health: `powershell -ExecutionPolicy Bypass -File .\\{}`\n- Windows stop: `powershell -ExecutionPolicy Bypass -File .\\{}`\n\n",
        manifest.scripts.preflight_windows,
        manifest.scripts.start_windows,
        manifest.scripts.health_windows,
        manifest.scripts.stop_windows
    ));
    text.push_str("## Generated Files\n\n");
    text.push_str("- `manifest.json`: launch pack inventory, runtime configuration summary, and SHA-256 artifact inventory.\n");
    text.push_str("- `start-order.txt`: deterministic node startup order.\n");
    text.push_str("- `wallet-provisioning.json`: structured wallet provisioning checklist with public keys and target paths only.\n");
    text.push_str("- `wallets/README.md`: local wallet directory instructions; place encrypted wallets here only when paths are relative to the pack.\n");
    text.push_str("- `validation-report.txt`: latest human-readable validation report.\n");
    text.push_str("- `validation-report.json`: latest structured validation report.\n\n");
    text.push_str("## Artifact Integrity\n\n");
    text.push_str(
        "`manifest.json` records SHA-256 values for generated configs, runbook, start-order, and platform scripts. Run `neo-nexus --validate-launch-pack .` after any operator edits to refresh validation evidence before handoff.\n\n",
    );
    text.push_str("## Secret Material Boundary\n\n");
    text.push_str(
        "This pack records public committee keys plus optional wallet, signer endpoint, and sidecar command template references only. It never includes private keys, wallet passwords, or generated genesis key material. `wallet-provisioning.json` is an operator checklist, not a wallet file and not a secret store.\n\n",
    );
    text.push_str(&format!(
        "Wallet provisioning policy: `{}`. Required wallets: `{}`. Existing wallet references: `{}`. Missing wallet references: `{}`. Generated secrets: `{}`.\n\n",
        manifest.secret_provisioning.policy,
        manifest.secret_provisioning.required_wallet_count,
        manifest.secret_provisioning.wallet_reference_count,
        manifest.secret_provisioning.missing_wallet_reference_count,
        manifest.secret_provisioning.generated_secret_count
    ));
    if !manifest.committee.signers.is_empty() {
        text.push_str("## Committee References\n\n");
        for signer in &manifest.committee.signers {
            let sidecar_plan = signer
                .signer_command_plan
                .as_ref()
                .map(|plan| {
                    format!(
                        "{}:{} args={}",
                        plan.execution_policy,
                        plan.binary,
                        plan.arguments.len()
                    )
                })
                .unwrap_or_else(|| "-".to_string());
            text.push_str(&format!(
                "- `{}` `{}` wallet={} endpoint={} sidecar_template={} sidecar={} sidecar_plan={}\n",
                signer.label,
                signer.public_key,
                signer.wallet_path.as_deref().unwrap_or("-"),
                signer.signer_endpoint.as_deref().unwrap_or("-"),
                signer.signer_command_template.as_deref().unwrap_or("-"),
                signer.signer_command.as_deref().unwrap_or("-"),
                sidecar_plan
            ));
        }
        text.push('\n');
    }
    text.push_str("## Node Start Order\n\n");
    for node in &manifest.nodes {
        text.push_str(&format!(
            "{}. `{}` `{}` RPC={} P2P={} WS={} config=`{}` config_sha256=`{}`\n",
            node.start_order,
            node.name,
            node.role,
            node.rpc_port,
            node.p2p_port,
            node.ws_port
                .map_or_else(|| "-".to_string(), |port| port.to_string()),
            node.config_path,
            node.config_sha256
        ));
    }
    text
}
