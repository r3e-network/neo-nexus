use anyhow::Result;

use super::{WorkspaceConfigExport, WorkspaceConfigReport};

impl WorkspaceConfigExport {
    pub fn to_cli_text(&self) -> String {
        let mut lines = vec![
            "node-config-export: ok".to_string(),
            format!("database: {}", self.report.database),
            format!("output-dir: {}", self.output_dir.display()),
            format!("nodes: {}", self.report.node_count),
            format!("files: {}", self.report.exported_file_count),
            format!("bytes-written: {}", self.report.total_bytes_written),
            format!("report-text: {}", self.text_path.display()),
            format!("report-json: {}", self.json_path.display()),
        ];

        if self.report.nodes.is_empty() {
            lines.push("file: none".to_string());
        } else {
            for node in &self.report.nodes {
                lines.push(format!(
                    "file: {} | {} | {} | {} | {} bytes",
                    node.node_name,
                    node.node_type,
                    node.config_format,
                    node.path,
                    node.bytes_written
                ));
            }
        }

        lines.push(String::new());
        lines.join("\n")
    }
}

impl WorkspaceConfigReport {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            "node-config-export: ok".to_string(),
            format!("application-version: {}", self.application_version),
            format!("generated-at-unix: {}", self.generated_at_unix),
            format!("database: {}", self.database),
            format!("output-dir: {}", self.output_dir),
            format!("nodes: {}", self.node_count),
            format!("files: {}", self.exported_file_count),
            format!("bytes-written: {}", self.total_bytes_written),
        ];

        if self.nodes.is_empty() {
            lines.push("file: none".to_string());
        } else {
            for node in &self.nodes {
                lines.push(format!(
                    "file: {} | {} | {} | rpc {} | p2p {} | plugins {}/{} | {}",
                    node.node_name,
                    node.node_type,
                    node.config_format,
                    node.rpc_port,
                    node.p2p_port,
                    node.enabled_plugin_count,
                    node.plugin_count,
                    node.path
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
