use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};

mod files;
mod findings;
mod report;
mod rules;

pub use report::{NativeUiAuditFinding, NativeUiAuditReport};

use self::{
    files::load_audit_files,
    findings::{forbidden_findings, required_findings},
    rules::{cargo_dependency_requirements, required_markers},
};

pub struct NativeUiAuditor;

impl NativeUiAuditor {
    pub fn audit(root: impl AsRef<Path>) -> Result<NativeUiAuditReport> {
        Self::audit_at(root, current_unix_time()?)
    }

    pub fn audit_at(root: impl AsRef<Path>, checked_at_unix: u64) -> Result<NativeUiAuditReport> {
        let root = root.as_ref();
        if !root.is_dir() {
            anyhow::bail!(
                "native UI root {} does not exist or is not a directory",
                root.display()
            );
        }
        let root = root
            .canonicalize()
            .with_context(|| format!("failed to resolve native UI root {}", root.display()))?;
        let files = load_audit_files(&root)?;
        let mut findings = forbidden_findings(&files);
        let mut missing_required = required_findings(&root, &files)?;

        findings.sort_by(|left, right| {
            left.path
                .cmp(&right.path)
                .then(left.category.cmp(&right.category))
                .then(left.marker.cmp(&right.marker))
        });
        missing_required.sort_by(|left, right| {
            left.path
                .cmp(&right.path)
                .then(left.category.cmp(&right.category))
                .then(left.marker.cmp(&right.marker))
        });

        let required_total = required_markers().len() + cargo_dependency_requirements().len();
        let required_passed = required_total.saturating_sub(missing_required.len());
        let forbidden_count = findings.len();
        let finding_count = missing_required.len() + findings.len();
        let status = if finding_count == 0 {
            "native"
        } else {
            "failed"
        }
        .to_string();

        Ok(NativeUiAuditReport {
            schema_version: 1,
            status,
            checked_at_unix,
            root_path: root,
            scanned_files: files.len(),
            required_total,
            required_passed,
            missing_required,
            forbidden_count,
            finding_count,
            findings,
        })
    }
}

fn current_unix_time() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time is before UNIX_EPOCH")?
        .as_secs())
}
