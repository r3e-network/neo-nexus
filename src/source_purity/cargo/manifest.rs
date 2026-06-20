use std::{fs, path::Path};

use anyhow::{Context, Result};

use super::{cargo_parse_error_finding, sort_and_dedup_findings};
use crate::source_purity::{rules::is_webview_cargo_package, SourcePurityFinding};

pub(super) fn cargo_manifest_dependency_findings(
    relative_path: &str,
    path: &Path,
) -> Result<Vec<SourcePurityFinding>> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read Cargo manifest {}", path.display()))?;
    let manifest = match toml::from_str::<toml::Value>(&text) {
        Ok(manifest) => manifest,
        Err(error) => {
            return Ok(vec![cargo_parse_error_finding(
                relative_path,
                "Cargo.toml",
                error,
            )]);
        }
    };

    let mut findings = Vec::new();
    collect_manifest_dependency_findings(relative_path, &manifest, &mut findings);
    sort_and_dedup_findings(&mut findings);
    Ok(findings)
}

fn collect_manifest_dependency_findings(
    relative_path: &str,
    value: &toml::Value,
    findings: &mut Vec<SourcePurityFinding>,
) {
    let Some(table) = value.as_table() else {
        return;
    };

    for (key, child) in table {
        if is_dependency_table_name(key) {
            collect_dependency_table_findings(relative_path, child, findings);
        }
        collect_manifest_dependency_findings(relative_path, child, findings);
    }
}

fn collect_dependency_table_findings(
    relative_path: &str,
    value: &toml::Value,
    findings: &mut Vec<SourcePurityFinding>,
) {
    let Some(dependencies) = value.as_table() else {
        return;
    };

    for (dependency_name, dependency_value) in dependencies {
        if is_webview_cargo_package(dependency_name) {
            findings.push(webview_manifest_finding(relative_path, dependency_name));
        }
        if let Some(package_name) = dependency_value
            .as_table()
            .and_then(|table| table.get("package"))
            .and_then(toml::Value::as_str)
        {
            if is_webview_cargo_package(package_name) {
                findings.push(webview_manifest_finding(relative_path, package_name));
            }
        }
    }
}

fn webview_manifest_finding(relative_path: &str, package_name: &str) -> SourcePurityFinding {
    SourcePurityFinding {
        path: relative_path.to_string(),
        category: "webview-cargo-dependency".to_string(),
        message: format!(
            "Cargo.toml depends on {package_name}, which would reintroduce a WebView/Tauri application shell"
        ),
    }
}

fn is_dependency_table_name(name: &str) -> bool {
    matches!(
        name,
        "dependencies" | "dev-dependencies" | "build-dependencies"
    )
}
