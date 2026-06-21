use super::*;

pub(super) const ARTIFACT_INTEGRITY: &[&str] = &[
    "`manifest.json` records SHA-256 values for generated configs, runbook, start-order, and platform scripts.",
    "Run `neo-nexus --validate-launch-pack .` after operator edits to refresh validation evidence before handoff.",
];

pub(super) const SECRET_MATERIAL_BOUNDARY: &[&str] = &[
    "This pack records public committee keys plus optional wallet, signer endpoint, and sidecar command references only.",
    "It never includes private keys, wallet passwords, or generated genesis key material.",
    "`wallet-provisioning.json` is an operator checklist, not a wallet file and not a secret store.",
];

const OPERATOR_FLOW: &[&str] = &[
    "Place required node binaries, config files, work directories, and signer wallet references on this host.",
    "Run `neo-nexus --validate-launch-pack .` from this directory and inspect validation reports.",
    "Run the platform preflight script.",
    "Run the platform start script.",
    "Run the platform health script until all nodes are ready.",
    "Run the platform stop script when shutting down the lab.",
];

const GENERATED_FILES: &[(&str, &str)] = &[
    (
        "manifest.json",
        "launch pack inventory, runtime configuration summary, and SHA-256 artifact inventory.",
    ),
    ("start-order.txt", "deterministic node startup order."),
    (
        "wallet-provisioning.json",
        "structured wallet provisioning checklist with public keys and target paths only.",
    ),
    (
        "wallets/README.md",
        "local wallet directory instructions for relative encrypted wallet paths.",
    ),
    (
        "validation-report.txt",
        "latest human-readable validation report.",
    ),
    (
        "validation-report.json",
        "latest structured validation report.",
    ),
];

pub(super) fn push_operator_flow(text: &mut String) {
    text.push_str("## Operator Flow\n\n");
    for (index, step) in OPERATOR_FLOW.iter().enumerate() {
        text.push_str(&format!("{}. {step}\n", index + 1));
    }
    text.push('\n');
}

pub(super) fn push_platform_commands(text: &mut String, scripts: &DeploymentScriptsManifest) {
    text.push_str("## Platform Commands\n\n");
    push_unix_commands(text, scripts);
    push_windows_commands(text, scripts);
    text.push('\n');
}

pub(super) fn push_generated_files(text: &mut String) {
    text.push_str("## Generated Files\n\n");
    for (file, description) in GENERATED_FILES {
        text.push_str(&format!("- `{file}`: {description}\n"));
    }
    text.push('\n');
}

pub(super) fn push_paragraph(text: &mut String, heading: &str, sentences: &[&str]) {
    text.push_str(heading);
    text.push_str("\n\n");
    text.push_str(&sentences.join(" "));
    text.push_str("\n\n");
}

fn push_unix_commands(text: &mut String, scripts: &DeploymentScriptsManifest) {
    push_command(
        text,
        "Unix/macOS",
        "preflight",
        &format!("./{}", scripts.preflight_unix),
    );
    push_command(
        text,
        "Unix/macOS",
        "start",
        &format!("./{}", scripts.start_unix),
    );
    push_command(
        text,
        "Unix/macOS",
        "health",
        &format!("./{}", scripts.health_unix),
    );
    push_command(
        text,
        "Unix/macOS",
        "stop",
        &format!("./{}", scripts.stop_unix),
    );
}

fn push_windows_commands(text: &mut String, scripts: &DeploymentScriptsManifest) {
    push_command(
        text,
        "Windows",
        "preflight",
        &windows_script_command(&scripts.preflight_windows),
    );
    push_command(
        text,
        "Windows",
        "start",
        &windows_script_command(&scripts.start_windows),
    );
    push_command(
        text,
        "Windows",
        "health",
        &windows_script_command(&scripts.health_windows),
    );
    push_command(
        text,
        "Windows",
        "stop",
        &windows_script_command(&scripts.stop_windows),
    );
}

fn push_command(text: &mut String, platform: &str, action: &str, command: &str) {
    text.push_str(&format!("- {platform} {action}: `{command}`\n"));
}

fn windows_script_command(script: &str) -> String {
    format!("powershell -ExecutionPolicy Bypass -File .\\{script}")
}
