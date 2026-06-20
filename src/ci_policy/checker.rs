use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use serde_yaml::Value;

use super::{model::CiPolicyReport, scan::build_report};

pub struct CiPolicyChecker;

impl CiPolicyChecker {
    pub fn check(path: impl AsRef<Path>) -> Result<CiPolicyReport> {
        Self::check_at(path, current_unix_time()?)
    }

    pub fn check_at(path: impl AsRef<Path>, checked_at_unix: u64) -> Result<CiPolicyReport> {
        let path = path.as_ref();
        if !path.is_file() {
            anyhow::bail!(
                "CI workflow {} does not exist; pass .github/workflows/ci.yml",
                path.display()
            );
        }
        let workflow_path = path
            .canonicalize()
            .with_context(|| format!("failed to resolve CI workflow {}", path.display()))?;
        let text = fs::read_to_string(&workflow_path)
            .with_context(|| format!("failed to read CI workflow {}", workflow_path.display()))?;
        Self::check_text_at(workflow_path, &text, checked_at_unix)
    }

    pub fn check_text_at(
        workflow_path: impl Into<PathBuf>,
        text: &str,
        checked_at_unix: u64,
    ) -> Result<CiPolicyReport> {
        let workflow_path = workflow_path.into();
        let yaml = serde_yaml::from_str::<Value>(text)
            .with_context(|| format!("failed to parse CI workflow {}", workflow_path.display()))?;
        Ok(build_report(workflow_path, text, &yaml, checked_at_unix))
    }
}

fn current_unix_time() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time is before UNIX_EPOCH")?
        .as_secs())
}
