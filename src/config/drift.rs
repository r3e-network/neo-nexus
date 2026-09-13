//! Configuration drift detection and atomic reconciliation.
//!
//! Compares physical on-disk configuration files against golden specifications
//! generated from the workspace repository, detects semantic and text divergence,
//! and safely reconciles drift with automatic backups.

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::types::{NodeConfig, NodeType};

use super::{export::StagedWrite, ConfigGenerator, ConfigValidationSeverity, ConfigValidator};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigDriftStatus {
    InSync,
    Drifted,
    Missing,
}

impl ConfigDriftStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::InSync => "in-sync",
            Self::Drifted => "drifted",
            Self::Missing => "missing",
        }
    }

    pub fn is_in_sync(self) -> bool {
        matches!(self, Self::InSync)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConfigDifference {
    pub category: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConfigDriftReport {
    pub node_name: String,
    pub node_type: NodeType,
    pub config_path: PathBuf,
    pub status: ConfigDriftStatus,
    pub differences: Vec<ConfigDifference>,
    pub expected_hash: String,
    pub actual_hash: Option<String>,
}

impl ConfigDriftReport {
    pub fn exit_code(&self) -> i32 {
        if self.status.is_in_sync() {
            0
        } else {
            1
        }
    }

    pub fn to_cli_text(&self) -> String {
        let mut lines = vec![
            format!("config-drift: {}", self.status.label()),
            format!("node: {}", self.node_name),
            format!("runtime: {}", self.node_type),
            format!("path: {}", self.config_path.display()),
            format!("expected-sha256: {}", self.expected_hash),
        ];
        if let Some(actual) = &self.actual_hash {
            lines.push(format!("actual-sha256: {actual}"));
        }
        if self.differences.is_empty() {
            lines.push("differences: none".to_string());
        } else {
            lines.push("differences:".to_string());
            for diff in &self.differences {
                lines.push(format!("  - [{}]: {}", diff.category, diff.detail));
            }
        }
        lines.join("\n")
    }
}

pub struct ConfigDriftDetector;

impl ConfigDriftDetector {
    /// Detects configuration drift between the node's expected configuration
    /// and the actual file currently stored on disk.
    pub fn check(node: &NodeConfig, config_path: &Path) -> Result<ConfigDriftReport> {
        let rendered = ConfigGenerator::render_for_node(node, &[])?;
        let expected_hash = sha256_hex(rendered.text.as_bytes());

        if !config_path.exists() {
            return Ok(ConfigDriftReport {
                node_name: node.name.clone(),
                node_type: node.node_type,
                config_path: config_path.to_path_buf(),
                status: ConfigDriftStatus::Missing,
                differences: vec![ConfigDifference {
                    category: "filesystem".to_string(),
                    detail: format!(
                        "configuration file does not exist at {}",
                        config_path.display()
                    ),
                }],
                expected_hash,
                actual_hash: None,
            });
        }

        let actual_text = fs::read_to_string(config_path)
            .with_context(|| format!("failed to read config file {}", config_path.display()))?;
        let actual_hash = sha256_hex(actual_text.as_bytes());

        if actual_hash == expected_hash {
            return Ok(ConfigDriftReport {
                node_name: node.name.clone(),
                node_type: node.node_type,
                config_path: config_path.to_path_buf(),
                status: ConfigDriftStatus::InSync,
                differences: Vec::new(),
                expected_hash,
                actual_hash: Some(actual_hash),
            });
        }

        let mut differences = Vec::new();
        differences.push(ConfigDifference {
            category: "content-hash".to_string(),
            detail: format!("hash divergence (expected {expected_hash}, actual {actual_hash})"),
        });

        let expected_line_count = rendered.text.lines().count();
        let actual_line_count = actual_text.lines().count();
        if expected_line_count != actual_line_count {
            differences.push(ConfigDifference {
                category: "line-count".to_string(),
                detail: format!(
                    "expected {expected_line_count} lines, actual file has {actual_line_count} lines"
                ),
            });
        }

        let validation = ConfigValidator::validate_text(node, rendered.format, &actual_text);
        for check in &validation.checks {
            if check.severity != ConfigValidationSeverity::Pass {
                differences.push(ConfigDifference {
                    category: "semantic-validation".to_string(),
                    detail: format!("{}: {}", check.title, check.detail),
                });
            }
        }

        Ok(ConfigDriftReport {
            node_name: node.name.clone(),
            node_type: node.node_type,
            config_path: config_path.to_path_buf(),
            status: ConfigDriftStatus::Drifted,
            differences,
            expected_hash,
            actual_hash: Some(actual_hash),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConfigReconciliationReport {
    pub node_name: String,
    pub config_path: PathBuf,
    pub reconciled: bool,
    pub backup_path: Option<PathBuf>,
    pub bytes_written: usize,
    pub post_check_status: ConfigDriftStatus,
}

impl ConfigReconciliationReport {
    pub fn to_cli_text(&self) -> String {
        let mut lines = vec![
            format!(
                "config-reconciliation: {}",
                if self.reconciled {
                    "applied"
                } else {
                    "already-in-sync"
                }
            ),
            format!("node: {}", self.node_name),
            format!("path: {}", self.config_path.display()),
            format!("status: {}", self.post_check_status.label()),
        ];
        if let Some(backup) = &self.backup_path {
            lines.push(format!("backup-saved-to: {}", backup.display()));
        }
        if self.reconciled {
            lines.push(format!("bytes-written: {}", self.bytes_written));
        }
        lines.join("\n")
    }
}

pub struct ConfigReconciler;

impl ConfigReconciler {
    /// Reconciles on-disk configuration with canonical generated configuration.
    ///
    /// Automatically backs up drifted physical files before performing an atomic
    /// write to ensure zero configuration loss.
    pub fn reconcile(
        node: &NodeConfig,
        config_path: &Path,
        create_backup: bool,
    ) -> Result<ConfigReconciliationReport> {
        let initial_check = ConfigDriftDetector::check(node, config_path)?;
        if initial_check.status == ConfigDriftStatus::InSync {
            return Ok(ConfigReconciliationReport {
                node_name: node.name.clone(),
                config_path: config_path.to_path_buf(),
                reconciled: false,
                backup_path: None,
                bytes_written: 0,
                post_check_status: ConfigDriftStatus::InSync,
            });
        }

        let mut backup_path = None;
        if create_backup && config_path.exists() {
            let timestamp = current_unix_time();
            let base_name = config_path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| "config".to_string());
            let backup_name = format!("{base_name}.drift-bak.{timestamp}");
            let target_backup = config_path.with_file_name(backup_name);
            fs::copy(config_path, &target_backup).with_context(|| {
                format!(
                    "failed to back up drifted config to {}",
                    target_backup.display()
                )
            })?;
            // `fs::copy` carries the source mode across, so a file written by
            // an older, wider reconcile would keep its permissions in a copy
            // that is never cleaned up. State the mode rather than inherit it.
            restrict_to_owner(&target_backup)?;
            backup_path = Some(target_backup);
        }

        let rendered = ConfigGenerator::render_for_node(node, &[])?;
        let contents = rendered.text.as_bytes();
        // Owner-only, matching `ConfigExporter`. Reconciling is a rewrite of the
        // same managed config, and writing it at the umask default silently
        // widened a 0600 file that can carry a wallet unlock password to 0644
        // — turning a drift repair into a permission downgrade.
        let staged = StagedWrite::new(config_path, contents, true)?;
        staged.commit()?;

        let post_check = ConfigDriftDetector::check(node, config_path)?;

        Ok(ConfigReconciliationReport {
            node_name: node.name.clone(),
            config_path: config_path.to_path_buf(),
            reconciled: true,
            backup_path,
            bytes_written: contents.len(),
            post_check_status: post_check.status,
        })
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn current_unix_time() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Restrict a managed config file to its owner, matching `ConfigExporter`.
#[cfg(unix)]
fn restrict_to_owner(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .with_context(|| format!("failed to restrict permissions for {}", path.display()))
}

/// Windows inherits the parent directory's ACL, which the workspace directory
/// already restricts; there is no mode to set here.
#[cfg(not(unix))]
fn restrict_to_owner(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/unit/config/drift/tests.rs"]
mod tests;
