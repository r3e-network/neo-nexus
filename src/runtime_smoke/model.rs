use std::path::PathBuf;

use serde::Serialize;

use crate::{preflight::RuntimeBinaryPreflight, types::NodeType};

use super::text::single_line;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeSmokeStatus {
    Passed,
    Review,
    Failed,
    TimedOut,
    Blocked,
}

impl RuntimeSmokeStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Review => "review",
            Self::Failed => "failed",
            Self::TimedOut => "timed-out",
            Self::Blocked => "blocked",
        }
    }

    pub fn is_success(self) -> bool {
        matches!(self, Self::Passed | Self::Review)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeSmokeAttempt {
    pub command_line: String,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub elapsed_ms: u128,
    pub stdout: String,
    pub stderr: String,
}

impl RuntimeSmokeAttempt {
    pub(super) fn output_text(&self) -> String {
        let mut text = String::new();
        text.push_str(&self.stdout);
        if !self.stderr.is_empty() {
            if !text.is_empty() {
                text.push('\n');
            }
            text.push_str(&self.stderr);
        }
        text
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeSmokeBinaryEvidenceStatus {
    Verified,
    Unavailable,
}

impl RuntimeSmokeBinaryEvidenceStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeSmokeBinaryEvidence {
    pub command_path: PathBuf,
    pub runtime_path: PathBuf,
    pub sha256: Option<String>,
    pub bytes: Option<u64>,
    pub status: RuntimeSmokeBinaryEvidenceStatus,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeSmokeReport {
    pub node_type: NodeType,
    pub binary_path: PathBuf,
    pub preflight: RuntimeBinaryPreflight,
    pub binary_evidence: RuntimeSmokeBinaryEvidence,
    pub status: RuntimeSmokeStatus,
    pub message: String,
    pub attempts: Vec<RuntimeSmokeAttempt>,
}

impl RuntimeSmokeReport {
    pub fn status_label(&self) -> &'static str {
        self.status.label()
    }

    pub fn operator_summary(&self) -> String {
        format!("runtime-smoke: {} ({})", self.status.label(), self.message)
    }

    pub fn to_cli_text(&self) -> String {
        let mut lines = vec![
            format!("runtime-smoke: {}", self.status.label()),
            format!("runtime: {}", self.node_type),
            format!("binary: {}", self.binary_path.display()),
            format!(
                "probe-command: {}",
                self.binary_evidence.command_path.display()
            ),
            format!(
                "runtime-binary: {}",
                self.binary_evidence.runtime_path.display()
            ),
            format!(
                "runtime-binary-evidence: {}",
                self.binary_evidence.status.label()
            ),
            format!("preflight: {}", self.preflight.status_label()),
            format!("message: {}", self.message),
        ];

        if let Some(bytes) = self.binary_evidence.bytes {
            lines.push(format!("runtime-binary-bytes: {bytes}"));
        }
        if let Some(sha256) = &self.binary_evidence.sha256 {
            lines.push(format!("runtime-binary-sha256: {sha256}"));
        }
        if self.binary_evidence.status == RuntimeSmokeBinaryEvidenceStatus::Unavailable {
            lines.push(format!(
                "runtime-binary-message: {}",
                self.binary_evidence.message
            ));
        }

        for (index, attempt) in self.attempts.iter().enumerate() {
            lines.push(format!("attempt-{}: {}", index + 1, attempt.command_line));
            lines.push(format!(
                "attempt-{}-result: exit={:?} timeout={} elapsed={}ms",
                index + 1,
                attempt.exit_code,
                attempt.timed_out,
                attempt.elapsed_ms
            ));
            if !attempt.stdout.trim().is_empty() {
                lines.push(format!(
                    "attempt-{}-stdout: {}",
                    index + 1,
                    single_line(&attempt.stdout)
                ));
            }
            if !attempt.stderr.trim().is_empty() {
                lines.push(format!(
                    "attempt-{}-stderr: {}",
                    index + 1,
                    single_line(&attempt.stderr)
                ));
            }
        }

        lines.push(String::new());
        lines.join("\n")
    }
}
