use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};

use super::{model::SourceQualityReport, scan::SourceQualityScan};

pub struct SourceQualityChecker;

impl SourceQualityChecker {
    pub fn check(root: impl AsRef<Path>) -> Result<SourceQualityReport> {
        Self::check_at(root, current_unix_time()?)
    }

    pub fn check_at(root: impl AsRef<Path>, checked_at_unix: u64) -> Result<SourceQualityReport> {
        let root = root.as_ref();
        if !root.is_dir() {
            anyhow::bail!(
                "source root {} does not exist or is not a directory",
                root.display()
            );
        }
        let root = root
            .canonicalize()
            .with_context(|| format!("failed to resolve source root {}", root.display()))?;
        let mut scan = SourceQualityScan::new(root.clone());
        scan.visit_dir(&root)?;
        scan.findings.sort_by(|left, right| {
            left.path
                .cmp(&right.path)
                .then(left.line.cmp(&right.line))
                .then(left.column.cmp(&right.column))
                .then(left.marker.cmp(&right.marker))
        });
        scan.skipped_directories.sort();
        let status = if scan.findings.is_empty() {
            "ok"
        } else {
            "failed"
        }
        .to_string();
        Ok(SourceQualityReport {
            schema_version: 1,
            status,
            checked_at_unix,
            root_path: root,
            scanned_files: scan.scanned_files,
            scanned_directories: scan.scanned_directories,
            skipped_directories: scan.skipped_directories,
            finding_count: scan.findings.len(),
            findings: scan.findings,
        })
    }
}

fn current_unix_time() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time is before UNIX_EPOCH")?
        .as_secs())
}
