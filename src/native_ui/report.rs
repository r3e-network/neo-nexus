use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeUiAuditReport {
    pub schema_version: u32,
    pub status: String,
    pub checked_at_unix: u64,
    pub root_path: PathBuf,
    pub scanned_files: usize,
    pub required_total: usize,
    pub required_passed: usize,
    pub missing_required: Vec<NativeUiAuditFinding>,
    pub forbidden_count: usize,
    pub finding_count: usize,
    pub findings: Vec<NativeUiAuditFinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeUiAuditFinding {
    pub path: String,
    pub marker: String,
    pub category: String,
    pub message: String,
}

impl NativeUiAuditReport {
    pub fn is_success(&self) -> bool {
        self.missing_required.is_empty() && self.findings.is_empty()
    }

    pub fn status_label(&self) -> &'static str {
        if self.is_success() {
            "native"
        } else {
            "failed"
        }
    }

    pub fn exit_code(&self) -> i32 {
        if self.is_success() {
            0
        } else {
            1
        }
    }

    pub fn to_cli_text(&self) -> String {
        let mut lines = vec![
            format!("native-ui-audit: {}", self.status_label()),
            format!("root: {}", self.root_path.display()),
            format!("scanned-files: {}", self.scanned_files),
            format!("required: {}/{}", self.required_passed, self.required_total),
            format!("forbidden: {}", self.forbidden_count),
            format!("findings: {}", self.finding_count),
        ];

        if self.missing_required.is_empty() && self.findings.is_empty() {
            lines.push("finding: none".to_string());
        } else {
            for finding in self.missing_required.iter().chain(self.findings.iter()) {
                lines.push(format!(
                    "finding: {} | {} | {} | {}",
                    finding.category, finding.path, finding.marker, finding.message
                ));
            }
        }

        lines.push(String::new());
        lines.join("\n")
    }
}
