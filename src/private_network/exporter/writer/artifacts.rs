use super::*;

pub(super) struct LaunchPackRenderedTexts {
    pub(super) start_order: String,
    pub(super) runbook: String,
    pub(super) wallet_provisioning: String,
    pub(super) wallet_instructions: String,
    pub(super) preflight_unix: String,
    pub(super) preflight_windows: String,
    pub(super) health_unix: String,
    pub(super) health_windows: String,
    pub(super) start_unix: String,
    pub(super) stop_unix: String,
    pub(super) start_windows: String,
    pub(super) stop_windows: String,
}

pub(super) fn render_launch_pack_texts(
    manifest: &DeploymentManifest,
) -> Result<LaunchPackRenderedTexts> {
    Ok(LaunchPackRenderedTexts {
        start_order: render_start_order(manifest),
        runbook: render_runbook(manifest),
        wallet_provisioning: render_wallet_provisioning(manifest)?,
        wallet_instructions: render_wallet_instructions(manifest),
        preflight_unix: render_unix_preflight_script(manifest),
        preflight_windows: render_windows_preflight_script(manifest),
        health_unix: render_unix_health_script(manifest),
        health_windows: render_windows_health_script(manifest),
        start_unix: render_unix_start_script(manifest),
        stop_unix: render_unix_stop_script(manifest),
        start_windows: render_windows_start_script(manifest),
        stop_windows: render_windows_stop_script(manifest),
    })
}

pub(super) fn attach_artifact_manifests(
    manifest: &mut DeploymentManifest,
    texts: &LaunchPackRenderedTexts,
) {
    manifest.artifacts = launch_pack_artifact_manifests(
        manifest,
        LaunchPackArtifactTexts {
            start_order: &texts.start_order,
            runbook: &texts.runbook,
            wallet_provisioning: &texts.wallet_provisioning,
            wallet_instructions: &texts.wallet_instructions,
            preflight_unix: &texts.preflight_unix,
            preflight_windows: &texts.preflight_windows,
            health_unix: &texts.health_unix,
            health_windows: &texts.health_windows,
            start_unix: &texts.start_unix,
            stop_unix: &texts.stop_unix,
            start_windows: &texts.start_windows,
            stop_windows: &texts.stop_windows,
        },
    );
}
