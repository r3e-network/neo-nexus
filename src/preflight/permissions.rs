use std::path::Path;

use super::model::{PreflightSeverity, RuntimePreflightCheck};

pub(super) fn permission_checks(
    binary_path: &Path,
    resolved_path: Option<&Path>,
) -> Vec<RuntimePreflightCheck> {
    let Some(resolved_path) = resolved_path else {
        return vec![RuntimePreflightCheck {
            severity: PreflightSeverity::Info,
            title: "Binary permission",
            detail: format!(
                "Executable permissions will be checked after {} resolves.",
                binary_path.display()
            ),
        }];
    };

    platform_permission_checks(resolved_path)
}

#[cfg(unix)]
fn platform_permission_checks(resolved_path: &Path) -> Vec<RuntimePreflightCheck> {
    use std::{fs, os::unix::fs::PermissionsExt};

    match fs::metadata(resolved_path) {
        Ok(metadata) if metadata.permissions().mode() & 0o111 != 0 => vec![RuntimePreflightCheck {
            severity: PreflightSeverity::Pass,
            title: "Binary permission",
            detail: "Executable bit is set for this host.".to_string(),
        }],
        Ok(_) => vec![RuntimePreflightCheck {
            severity: PreflightSeverity::Critical,
            title: "Binary permission",
            detail: format!(
                "{} is not executable on this host.",
                resolved_path.display()
            ),
        }],
        Err(error) => vec![RuntimePreflightCheck {
            severity: PreflightSeverity::Critical,
            title: "Binary permission",
            detail: format!(
                "Could not inspect {} permissions: {error}",
                resolved_path.display()
            ),
        }],
    }
}

#[cfg(not(unix))]
fn platform_permission_checks(_resolved_path: &Path) -> Vec<RuntimePreflightCheck> {
    vec![RuntimePreflightCheck {
        severity: PreflightSeverity::Pass,
        title: "Binary permission",
        detail: "Host does not expose Unix executable bits.".to_string(),
    }]
}
