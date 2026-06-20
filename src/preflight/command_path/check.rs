use std::{fs, path::Path};

use crate::preflight::model::{PreflightSeverity, RuntimePreflightCheck};

use super::{search::should_search_path, size::format_size};

pub(in crate::preflight) fn path_check(
    binary_path: &Path,
    resolved_path: Option<&Path>,
) -> RuntimePreflightCheck {
    if binary_path.as_os_str().is_empty() {
        return RuntimePreflightCheck {
            severity: PreflightSeverity::Critical,
            title: "Binary path",
            detail: "Binary path is empty.".to_string(),
        };
    }

    if let Some(resolved_path) = resolved_path {
        return resolved_binary_path_check(binary_path, resolved_path);
    }

    unresolved_binary_path_check(binary_path)
}

fn resolved_binary_path_check(binary_path: &Path, resolved_path: &Path) -> RuntimePreflightCheck {
    let size = fs::metadata(resolved_path)
        .map(|metadata| format_size(metadata.len()))
        .unwrap_or_else(|_| "unknown size".to_string());
    let detail = if resolved_path == binary_path {
        format!(
            "Executable file exists at {} ({size}).",
            resolved_path.display()
        )
    } else {
        format!(
            "{} resolves through PATH to {} ({size}).",
            binary_path.display(),
            resolved_path.display()
        )
    };

    RuntimePreflightCheck {
        severity: PreflightSeverity::Pass,
        title: "Binary path",
        detail,
    }
}

fn unresolved_binary_path_check(binary_path: &Path) -> RuntimePreflightCheck {
    RuntimePreflightCheck {
        severity: PreflightSeverity::Critical,
        title: "Binary path",
        detail: unresolved_binary_path_detail(binary_path),
    }
}

fn unresolved_binary_path_detail(binary_path: &Path) -> String {
    if binary_path.exists() {
        format!("{} exists but is not a file.", binary_path.display())
    } else if should_search_path(binary_path) {
        format!("{} was not found on PATH.", binary_path.display())
    } else {
        format!("{} was not found.", binary_path.display())
    }
}
