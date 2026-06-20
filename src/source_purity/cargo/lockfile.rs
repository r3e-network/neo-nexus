use std::{fs, path::Path};

use anyhow::{Context, Result};

use super::{cargo_parse_error_finding, sort_and_dedup_findings};
use crate::source_purity::{rules::is_webview_cargo_package, SourcePurityFinding};

pub(super) fn cargo_lock_dependency_findings(
    relative_path: &str,
    path: &Path,
) -> Result<Vec<SourcePurityFinding>> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read Cargo lockfile {}", path.display()))?;
    let lock = match toml::from_str::<toml::Value>(&text) {
        Ok(lock) => lock,
        Err(error) => {
            return Ok(vec![cargo_parse_error_finding(
                relative_path,
                "Cargo.lock",
                error,
            )])
        }
    };

    let mut findings = Vec::new();
    if let Some(packages) = lock.get("package").and_then(toml::Value::as_array) {
        for package in packages {
            collect_lock_package_finding(relative_path, package, &mut findings);
        }
    }
    sort_and_dedup_findings(&mut findings);
    Ok(findings)
}

fn collect_lock_package_finding(
    relative_path: &str,
    package: &toml::Value,
    findings: &mut Vec<SourcePurityFinding>,
) {
    let Some(name) = package
        .as_table()
        .and_then(|table| table.get("name"))
        .and_then(toml::Value::as_str)
    else {
        return;
    };

    if is_webview_cargo_package(name) {
        findings.push(SourcePurityFinding {
            path: relative_path.to_string(),
            category: "webview-cargo-lock-package".to_string(),
            message: format!(
                "Cargo.lock contains {name}, which indicates a WebView/Tauri dependency graph"
            ),
        });
    }
}
