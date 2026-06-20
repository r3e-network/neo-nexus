use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CiPolicyReport {
    pub schema_version: u32,
    pub status: String,
    pub checked_at_unix: u64,
    pub workflow_path: PathBuf,
    pub required_os: Vec<String>,
    pub found_os: Vec<String>,
    pub missing_os: Vec<String>,
    pub required_commands: Vec<String>,
    pub missing_commands: Vec<String>,
    pub forbidden_count: usize,
    pub finding_count: usize,
    pub findings: Vec<CiPolicyFinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CiPolicyFinding {
    pub category: String,
    pub marker: String,
    pub message: String,
}

impl CiPolicyReport {
    pub fn is_success(&self) -> bool {
        self.findings.is_empty()
    }

    pub fn status_label(&self) -> &'static str {
        if self.is_success() {
            "native-ci"
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
            format!("ci-policy: {}", self.status_label()),
            format!("workflow: {}", self.workflow_path.display()),
            format!("required-os: {}", self.required_os.join(", ")),
            format!("found-os: {}", value_or_none(&self.found_os)),
            format!("missing-os: {}", value_or_none(&self.missing_os)),
            format!("required-commands: {}", self.required_commands.len()),
            format!("missing-commands: {}", self.missing_commands.len()),
            format!("forbidden: {}", self.forbidden_count),
            format!("findings: {}", self.finding_count),
        ];

        if self.findings.is_empty() {
            lines.push("finding: none".to_string());
        } else {
            for finding in &self.findings {
                lines.push(format!(
                    "finding: {} | {} | {}",
                    finding.category, finding.marker, finding.message
                ));
            }
        }

        lines.push(String::new());
        lines.join("\n")
    }
}

fn value_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}
