use std::path::{Path, PathBuf};

use super::{artifacts::LaunchPackRenderedTexts, *};

pub(super) struct LaunchPackFileWriteReport {
    pub(super) manifest_path: PathBuf,
    pub(super) start_order_path: PathBuf,
    pub(super) runbook_path: PathBuf,
    pub(super) wallet_provisioning_path: PathBuf,
    pub(super) wallet_instructions_path: PathBuf,
    pub(super) preflight_unix_path: PathBuf,
    pub(super) preflight_windows_path: PathBuf,
    pub(super) health_unix_path: PathBuf,
    pub(super) health_windows_path: PathBuf,
    pub(super) start_unix_path: PathBuf,
    pub(super) stop_unix_path: PathBuf,
    pub(super) start_windows_path: PathBuf,
    pub(super) stop_windows_path: PathBuf,
    pub(super) bytes_written: usize,
}

pub(super) fn write_launch_pack_files(
    root_path: &Path,
    manifest: &DeploymentManifest,
    texts: &LaunchPackRenderedTexts,
) -> Result<LaunchPackFileWriteReport> {
    let mut bytes_written = 0;

    let manifest_text =
        serde_json::to_string_pretty(manifest).context("failed to render launch manifest")?;
    let manifest_path = root_path.join("manifest.json");
    write_text_file(&manifest_path, &manifest_text, "private network manifest")?;
    bytes_written += manifest_text.len();

    let start_order_path = root_path.join(START_ORDER_FILE);
    write_text_file(
        &start_order_path,
        &texts.start_order,
        "private network start order",
    )?;
    bytes_written += texts.start_order.len();

    let runbook_path = root_path.join("RUNBOOK.md");
    write_text_file(&runbook_path, &texts.runbook, "private network runbook")?;
    bytes_written += texts.runbook.len();

    let wallet_provisioning_path = root_path.join(WALLET_PROVISIONING_FILE);
    write_text_file(
        &wallet_provisioning_path,
        &texts.wallet_provisioning,
        "private network wallet provisioning plan",
    )?;
    bytes_written += texts.wallet_provisioning.len();

    let wallet_instructions_path = root_path.join(WALLET_INSTRUCTIONS_FILE);
    write_text_file(
        &wallet_instructions_path,
        &texts.wallet_instructions,
        "private network wallet instructions",
    )?;
    bytes_written += texts.wallet_instructions.len();

    let preflight_unix_path = root_path.join("preflight-unix.sh");
    write_script(&preflight_unix_path, &texts.preflight_unix, true)?;
    bytes_written += texts.preflight_unix.len();

    let preflight_windows_path = root_path.join("preflight-windows.ps1");
    write_script(&preflight_windows_path, &texts.preflight_windows, false)?;
    bytes_written += texts.preflight_windows.len();

    let health_unix_path = root_path.join("health-unix.sh");
    write_script(&health_unix_path, &texts.health_unix, true)?;
    bytes_written += texts.health_unix.len();

    let health_windows_path = root_path.join("health-windows.ps1");
    write_script(&health_windows_path, &texts.health_windows, false)?;
    bytes_written += texts.health_windows.len();

    let start_unix_path = root_path.join("start-unix.sh");
    write_script(&start_unix_path, &texts.start_unix, true)?;
    bytes_written += texts.start_unix.len();

    let stop_unix_path = root_path.join("stop-unix.sh");
    write_script(&stop_unix_path, &texts.stop_unix, true)?;
    bytes_written += texts.stop_unix.len();

    let start_windows_path = root_path.join("start-windows.ps1");
    write_script(&start_windows_path, &texts.start_windows, false)?;
    bytes_written += texts.start_windows.len();

    let stop_windows_path = root_path.join("stop-windows.ps1");
    write_script(&stop_windows_path, &texts.stop_windows, false)?;
    bytes_written += texts.stop_windows.len();

    Ok(LaunchPackFileWriteReport {
        manifest_path,
        start_order_path,
        runbook_path,
        wallet_provisioning_path,
        wallet_instructions_path,
        preflight_unix_path,
        preflight_windows_path,
        health_unix_path,
        health_windows_path,
        start_unix_path,
        stop_unix_path,
        start_windows_path,
        stop_windows_path,
        bytes_written,
    })
}
