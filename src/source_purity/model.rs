use std::path::PathBuf;

use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourcePurityReport {
    pub schema_version: u32,
    pub status: String,
    pub checked_at_unix: u64,
    pub root_path: PathBuf,
    pub scanned_files: usize,
    pub scanned_directories: usize,
    pub skipped_directories: Vec<String>,
    pub disallowed_count: usize,
    pub disallowed_entries: Vec<SourcePurityFinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourcePurityFinding {
    pub path: String,
    pub category: String,
    pub message: String,
}

impl SourcePurityReport {
    pub fn is_success(&self) -> bool {
        self.disallowed_entries.is_empty()
    }

    pub fn status_label(&self) -> &'static str {
        if self.is_success() {
            "pure-rust"
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
            format!("source-purity: {}", self.status_label()),
            format!("root: {}", self.root_path.display()),
            format!("scanned-files: {}", self.scanned_files),
            format!("scanned-directories: {}", self.scanned_directories),
            format!("skipped-directories: {}", self.skipped_directories.len()),
            format!("disallowed: {}", self.disallowed_count),
        ];

        if self.disallowed_entries.is_empty() {
            lines.push("finding: none".to_string());
        } else {
            for finding in &self.disallowed_entries {
                lines.push(format!(
                    "finding: {} | {} | {}",
                    finding.category, finding.path, finding.message
                ));
            }
        }

        lines.push(String::new());
        lines.join("\n")
    }

    pub fn to_json_text(&self) -> Result<String> {
        Ok(format!("{}\n", serde_json::to_string_pretty(self)?))
    }
}
