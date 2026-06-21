use super::{
    runbook_sections::{
        push_generated_files, push_operator_flow, push_paragraph, push_platform_commands,
        ARTIFACT_INTEGRITY, SECRET_MATERIAL_BOUNDARY,
    },
    *,
};

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
    push_operator_flow(&mut text);
    push_platform_commands(&mut text, &manifest.scripts);
    push_generated_files(&mut text);
    push_paragraph(&mut text, "## Artifact Integrity", ARTIFACT_INTEGRITY);
    push_paragraph(
        &mut text,
        "## Secret Material Boundary",
        SECRET_MATERIAL_BOUNDARY,
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
