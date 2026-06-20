use super::*;

pub(in crate::private_network) struct LaunchPackArtifactTexts<'a> {
    pub(in crate::private_network) start_order: &'a str,
    pub(in crate::private_network) runbook: &'a str,
    pub(in crate::private_network) wallet_provisioning: &'a str,
    pub(in crate::private_network) wallet_instructions: &'a str,
    pub(in crate::private_network) preflight_unix: &'a str,
    pub(in crate::private_network) preflight_windows: &'a str,
    pub(in crate::private_network) health_unix: &'a str,
    pub(in crate::private_network) health_windows: &'a str,
    pub(in crate::private_network) start_unix: &'a str,
    pub(in crate::private_network) stop_unix: &'a str,
    pub(in crate::private_network) start_windows: &'a str,
    pub(in crate::private_network) stop_windows: &'a str,
}

pub(in crate::private_network) fn launch_pack_artifact_manifests(
    manifest: &DeploymentManifest,
    texts: LaunchPackArtifactTexts<'_>,
) -> Vec<DeploymentArtifactManifest> {
    [
        ("start-order", START_ORDER_FILE, texts.start_order),
        ("runbook", manifest.scripts.runbook.as_str(), texts.runbook),
        (
            "wallet-provisioning",
            manifest
                .secret_provisioning
                .wallet_provisioning_file
                .as_str(),
            texts.wallet_provisioning,
        ),
        (
            "wallet-instructions",
            manifest
                .secret_provisioning
                .wallet_instructions_file
                .as_str(),
            texts.wallet_instructions,
        ),
        (
            "preflight-unix",
            manifest.scripts.preflight_unix.as_str(),
            texts.preflight_unix,
        ),
        (
            "preflight-windows",
            manifest.scripts.preflight_windows.as_str(),
            texts.preflight_windows,
        ),
        (
            "health-unix",
            manifest.scripts.health_unix.as_str(),
            texts.health_unix,
        ),
        (
            "health-windows",
            manifest.scripts.health_windows.as_str(),
            texts.health_windows,
        ),
        (
            "start-unix",
            manifest.scripts.start_unix.as_str(),
            texts.start_unix,
        ),
        (
            "stop-unix",
            manifest.scripts.stop_unix.as_str(),
            texts.stop_unix,
        ),
        (
            "start-windows",
            manifest.scripts.start_windows.as_str(),
            texts.start_windows,
        ),
        (
            "stop-windows",
            manifest.scripts.stop_windows.as_str(),
            texts.stop_windows,
        ),
    ]
    .into_iter()
    .map(|(label, path, text)| DeploymentArtifactManifest {
        label: label.to_string(),
        path: path.to_string(),
        sha256: sha256_bytes(text.as_bytes()),
        bytes: text.len() as u64,
    })
    .collect()
}
