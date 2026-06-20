use anyhow::Result;

use super::model::WorkspaceReadinessReport;

impl WorkspaceReadinessReport {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("workspace-readiness-report: {}", self.status),
            format!("application-version: {}", self.application_version),
            format!("generated-at-unix: {}", self.generated_at_unix),
            format!("database: {}", self.database),
            format!("score: {}", self.score),
            format!("nodes: {}/{}", self.ready_nodes, self.node_count),
            format!("critical: {}", self.critical_count),
            format!("warnings: {}", self.warning_count),
        ];

        if self.findings.is_empty() {
            lines.push("finding: none".to_string());
        } else {
            for finding in &self.findings {
                lines.push(format!(
                    "finding: {} | {} | {} | {}",
                    finding.severity, finding.node_name, finding.title, finding.detail
                ));
            }
        }

        lines.push(String::new());
        for node in &self.nodes {
            lines.push(format!(
                "node: {} | {} | score={} | critical={} | warnings={}",
                node.status, node.node_name, node.score, node.critical_count, node.warning_count
            ));
            for check in &node.checks {
                lines.push(format!(
                    "check: {} | {} | {}",
                    check.severity, check.title, check.detail
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
