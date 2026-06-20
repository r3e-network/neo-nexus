use std::path::Path;

use super::ConfigValidationReport;
use crate::config::validation::ConfigValidationSeverity;

impl ConfigValidationReport {
    pub fn to_cli_text(&self, source_path: &Path) -> String {
        let mut lines = vec![
            format!("node-config-validation: {}", self.status_label()),
            format!("runtime: {}", self.node_type),
            format!("format: {}", self.format.label()),
            format!("source: {}", source_path.display()),
            format!("checks: {}", self.summary()),
        ];

        let mut finding_count = 0;
        for severity in [
            ConfigValidationSeverity::Critical,
            ConfigValidationSeverity::Warning,
        ] {
            for check in self
                .checks
                .iter()
                .filter(|check| check.severity == severity)
            {
                lines.push(format!(
                    "finding: {} | {} | {}",
                    severity.label(),
                    check.title,
                    check.detail
                ));
                finding_count += 1;
            }
        }

        if finding_count == 0 {
            lines.push("finding: none".to_string());
        }
        lines.push(String::new());
        lines.join("\n")
    }
}
