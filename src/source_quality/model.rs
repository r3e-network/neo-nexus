use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceQualityReport {
    pub schema_version: u32,
    pub status: String,
    pub checked_at_unix: u64,
    pub root_path: PathBuf,
    pub scanned_files: usize,
    pub rust_files: usize,
    pub maintenance_files: usize,
    pub scanned_directories: usize,
    pub skipped_directories: Vec<String>,
    pub finding_count: usize,
    pub findings: Vec<SourceQualityFinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceQualityFinding {
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub marker: String,
    pub category: String,
    pub snippet: String,
    pub hint: String,
}

impl SourceQualityReport {
    pub fn is_success(&self) -> bool {
        self.findings.is_empty()
    }

    pub fn status_label(&self) -> &'static str {
        if self.is_success() {
            "ok"
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
            format!("source-quality: {}", self.status_label()),
            format!("root: {}", self.root_path.display()),
            format!("scanned-files: {}", self.scanned_files),
            format!("rust-files: {}", self.rust_files),
            format!("maintenance-files: {}", self.maintenance_files),
            format!("scanned-directories: {}", self.scanned_directories),
            format!("skipped-directories: {}", self.skipped_directories.len()),
            format!("findings: {}", self.finding_count),
        ];

        if self.findings.is_empty() {
            lines.push("finding: none".to_string());
        } else {
            for finding in &self.findings {
                lines.push(format!(
                    "finding: {}:{}:{} | {} | {} | snippet: {} | hint: {}",
                    finding.path,
                    finding.line,
                    finding.column,
                    finding.category,
                    finding.marker,
                    finding.snippet,
                    finding.hint
                ));
            }
        }

        lines.push(String::new());
        lines.join("\n")
    }
}
