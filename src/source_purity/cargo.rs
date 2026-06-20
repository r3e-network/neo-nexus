mod lockfile;
mod manifest;

use std::{fmt::Display, path::Path};

use anyhow::Result;

use super::SourcePurityFinding;

use self::{
    lockfile::cargo_lock_dependency_findings, manifest::cargo_manifest_dependency_findings,
};

pub(super) fn cargo_dependency_findings(
    name: &str,
    relative_path: &str,
    path: &Path,
) -> Result<Vec<SourcePurityFinding>> {
    match name {
        "Cargo.toml" => cargo_manifest_dependency_findings(relative_path, path),
        "Cargo.lock" => cargo_lock_dependency_findings(relative_path, path),
        _ => Ok(Vec::new()),
    }
}

pub(in crate::source_purity::cargo) fn cargo_parse_error_finding(
    relative_path: &str,
    artifact: &str,
    error: impl Display,
) -> SourcePurityFinding {
    SourcePurityFinding {
        path: relative_path.to_string(),
        category: "cargo-metadata-parse-error".to_string(),
        message: format!("failed to parse {artifact} during native-boundary audit: {error}"),
    }
}

pub(in crate::source_purity::cargo) fn sort_and_dedup_findings(
    findings: &mut Vec<SourcePurityFinding>,
) {
    findings.sort_by(|left, right| left.message.cmp(&right.message));
    findings.dedup_by(|left, right| {
        left.path == right.path && left.category == right.category && left.message == right.message
    });
}
