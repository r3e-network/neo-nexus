use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use super::super::{VALIDATION_REPORT_JSON_FILE, VALIDATION_REPORT_TEXT_FILE};
use super::checks::LaunchPackValidationCheck;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PrivateNetworkLaunchPackValidation {
    pub root_path: PathBuf,
    pub manifest_path: PathBuf,
    pub schema_version: u32,
    pub node_count: usize,
    pub signer_count: usize,
    pub passed_count: usize,
    pub warning_count: usize,
    pub failed_count: usize,
    pub checks: Vec<LaunchPackValidationCheck>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateNetworkLaunchPackValidationReport {
    pub text_path: PathBuf,
    pub json_path: PathBuf,
    pub bytes_written: usize,
}

impl PrivateNetworkLaunchPackValidation {
    pub fn is_success(&self) -> bool {
        self.failed_count == 0
    }

    pub fn to_cli_text(&self) -> String {
        let status = if self.is_success() { "ok" } else { "failed" };
        let mut text = format!(
            "launch-pack: {status}\nmanifest: {}\nschema: {}\nnodes: {}\nsigners: {}\nchecks: {} passed, {} warnings, {} failed\n",
            self.manifest_path.display(),
            self.schema_version,
            self.node_count,
            self.signer_count,
            self.passed_count,
            self.warning_count,
            self.failed_count
        );
        for check in &self.checks {
            text.push_str(&format!(
                "{} [{}] {}: {}\n",
                check.status.label(),
                check.category,
                check.label,
                check.message
            ));
        }
        text
    }

    pub fn write_reports(&self) -> Result<PrivateNetworkLaunchPackValidationReport> {
        fs::create_dir_all(&self.root_path).with_context(|| {
            format!(
                "failed to create validation report directory {}",
                self.root_path.display()
            )
        })?;
        let text = self.to_cli_text();
        let text_path = self.root_path.join(VALIDATION_REPORT_TEXT_FILE);
        fs::write(&text_path, text.as_bytes()).with_context(|| {
            format!(
                "failed to write launch pack validation report {}",
                text_path.display()
            )
        })?;

        let json = serde_json::to_string_pretty(self)
            .context("failed to render launch pack validation JSON report")?;
        let json_path = self.root_path.join(VALIDATION_REPORT_JSON_FILE);
        fs::write(&json_path, json.as_bytes()).with_context(|| {
            format!(
                "failed to write launch pack validation report {}",
                json_path.display()
            )
        })?;

        Ok(PrivateNetworkLaunchPackValidationReport {
            text_path,
            json_path,
            bytes_written: text.len() + json.len(),
        })
    }
}
