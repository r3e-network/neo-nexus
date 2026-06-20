use super::super::super::super::*;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub(super) fn executable_file_status(
    path: &Path,
    pass_message: String,
) -> (LaunchPackValidationStatus, String) {
    if !path.exists() {
        return (
            LaunchPackValidationStatus::Fail,
            format!("sidecar binary missing: {}", path.display()),
        );
    }
    if !path.is_file() {
        return (
            LaunchPackValidationStatus::Fail,
            format!("sidecar binary is not a file: {}", path.display()),
        );
    }
    if !host_can_execute_file(path) {
        return (
            LaunchPackValidationStatus::Fail,
            format!(
                "sidecar binary is not executable on this host: {}",
                path.display()
            ),
        );
    }
    (LaunchPackValidationStatus::Pass, pass_message)
}

#[cfg(unix)]
fn host_can_execute_file(path: &Path) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn host_can_execute_file(path: &Path) -> bool {
    path.is_file()
}
