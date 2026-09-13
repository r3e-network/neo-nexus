//! Metrics exporter adapter for neo-cli
//!
//! neo-cli uses external prometheus-net-adapter library to expose metrics.
//! This adapter manages the sidecar process and normalizes metrics output.

use crate::config::GenerationContext;
use crate::supervisor::model::{
    FatalError, LogEntry, LogParserAdapter, MetricsExporterAdapter, PluginMetadata,
    PluginSystemAdapter, SyncProgress,
};
use crate::types::NodeConfig;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// External process managing prometheus metrics for neo-cli
#[derive(Debug, Clone)]
pub struct NeoCliMetricsExporter {
    /// PID of the sidecar exporter process
    pub pid: Option<u32>,
    /// Port where metrics are exposed
    pub port: u16,
    /// Path to the configuration file
    pub config_path: PathBuf,
}

impl NeoCliMetricsExporter {
    pub fn new(node_dir: &Path) -> Self {
        let port = 9090; // Default prometheus port for neo-cli
        let config_path = node_dir.join("prometheus.json");

        Self {
            pid: None,
            port,
            config_path,
        }
    }

    /// Start the prometheus-net-adapter sidecar process
    pub fn start_sidecar(&mut self, binary_path: &Path) -> Result<()> {
        let cmd = Command::new(binary_path)
            .arg("--config")
            .arg(&self.config_path)
            .arg("--port")
            .arg(self.port.to_string())
            .spawn()
            .context("Failed to start neo-cli prometheus sidecar")?;

        self.pid = Some(cmd.id());
        Ok(())
    }

    /// Stop the sidecar process gracefully
    pub fn stop(&self) -> Result<()> {
        if let Some(pid) = self.pid {
            // Use taskkill on Windows, kill on Unix
            #[cfg(windows)]
            Command::new("taskkill")
                .arg("/PID")
                .arg(pid.to_string())
                .arg("/F")
                .output()?;

            #[cfg(not(windows))]
            Command::new("kill")
                .arg("-TERM")
                .arg(pid.to_string())
                .output()?;
        }
        Ok(())
    }

    /// Fetch metrics from the sidecar endpoint
    pub fn fetch_metrics(&self) -> Result<String> {
        use reqwest::blocking::Client;
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()?;
        let url = format!("http://localhost:{}/metrics", self.port);

        let response = client.get(&url).send()?;
        Ok(response.text()?)
    }
}

impl MetricsExporterAdapter for NeoCliMetricsExporter {
    fn exporter_package(&self) -> Option<&'static str> {
        Some("prometheus-net-adapter")
    }

    fn metrics_url(&self, _rpc_port: u16) -> Option<String> {
        Some(format!("http://localhost:{}/metrics", self.port))
    }

    fn generate_config(&self, _node: &NodeConfig) -> Result<Vec<u8>> {
        // Generate JSON config for neo-cli prometheus adapter
        let config = serde_json::json!({
            "listen_address": format!(":{}", self.port),
            "path": "/metrics",
            "namespace": "neo_cli",
            "subsystem": "node"
        });

        Ok(serde_json::to_vec_pretty(&config)?)
    }

    fn normalize_metrics(&self, raw: &[u8]) -> Result<String> {
        // neo-cli prometheus output is already in standard format
        // Add Neo-specific labels if needed
        let mut normalized = String::from_utf8_lossy(raw).to_string();

        // Ensure proper exposition format
        if !normalized.ends_with('\n') {
            normalized.push('\n');
        }

        Ok(normalized)
    }
}

impl LogParserAdapter for NeoCliMetricsExporter {
    fn parse_line(&self, line: &str) -> Option<LogEntry> {
        // neo-cli logs are often JSON-formatted or key=value pairs
        // Try parsing as JSON first
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            return Some(LogEntry {
                timestamp: json.get("timestamp").and_then(|v| v.as_u64()).unwrap_or(0),
                level: json
                    .get("level")
                    .and_then(|v| v.as_str())
                    .unwrap_or("info")
                    .to_string(),
                message: json
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or(line)
                    .to_string(),
                source: json
                    .get("source")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                metadata: json,
            });
        }

        // Fall back to key=value parsing
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            Some(LogEntry {
                timestamp: 0,
                level: "info".to_string(),
                message: parts[1..].join(" "),
                source: None,
                metadata: serde_json::json!({"raw": line}),
            })
        } else {
            None
        }
    }

    fn detect_fatal_errors(&self, log_content: &str) -> Vec<FatalError> {
        let mut errors = Vec::new();

        let fatal_patterns = [
            (
                "consensus failure",
                "Consensus module encountered a critical error",
            ),
            ("rpc error", "RPC server failed to initialize"),
            (
                "database error",
                "Storage backend encountered a fatal error",
            ),
            ("sync failed", "Blockchain synchronization failed"),
        ];

        for (pattern, suggestion) in &fatal_patterns {
            if log_content.to_lowercase().contains(pattern) {
                errors.push(FatalError {
                    line_number: 0, // Would be calculated in real usage
                    pattern: (*pattern).to_string(),
                    suggestion: suggestion.to_string(),
                });
            }
        }

        errors
    }

    fn extract_sync_progress(&self, lines: &[&str]) -> Option<SyncProgress> {
        // Look for sync progress patterns in neo-cli logs
        // Example: "Block #12345/100000 synced at 15 MB/s"

        for line in lines.iter().rev().take(10) {
            if let Some(block_num) = self.parse_block_number(line) {
                return Some(SyncProgress {
                    current_height: block_num,
                    target_height: 10_429_678, // Mainnet height (placeholder)
                    sync_percentage: (block_num as f32 / 10_429_678.0) * 100.0,
                    peers_connected: 0, // Would need network stats
                });
            }
        }

        None
    }
}

impl PluginSystemAdapter for NeoCliMetricsExporter {
    fn discover_plugins(&self, node_dir: &Path) -> Result<Vec<PluginMetadata>> {
        let plugin_dir = node_dir.join("Plugins");
        if !plugin_dir.exists() {
            return Ok(Vec::new());
        }

        let mut plugins = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&plugin_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("dll") {
                    let name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    plugins.push(PluginMetadata {
                        id: name.clone(),
                        name: name.clone(),
                        version: "unknown".to_string(),
                        enabled: true, // Would check manifest in real usage
                    });
                }
            }
        }

        Ok(plugins)
    }

    fn install_plugin(&self, plugin_id: &str, target_dir: &Path) -> Result<()> {
        // Copy DLL to Plugins directory
        let plugin_dir = target_dir.join("Plugins");
        std::fs::create_dir_all(&plugin_dir)?;

        // In real usage, would download from catalog and copy binary
        // For now, just create placeholder manifest
        let manifest = plugin_dir.join(format!("{}.manifest.json", plugin_id));
        std::fs::write(
            &manifest,
            serde_json::to_string_pretty(&serde_json::json!({
                "id": plugin_id,
                "version": "1.0.0",
                "enabled": true
            }))?,
        )?;

        Ok(())
    }

    fn toggle_plugin(&self, plugin_id: &str, enabled: bool, ctx: &GenerationContext) -> Result<()> {
        // Update the manifest file to change enabled state
        let plugin_dir = super::node_dir_from_context(ctx)?.join("Plugins");
        let manifest = plugin_dir.join(format!("{}.manifest.json", plugin_id));

        if manifest.exists() {
            let mut manifest_data: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(&manifest)?)?;

            if let Some(obj) = manifest_data.as_object_mut() {
                obj.insert("enabled".to_string(), serde_json::json!(enabled));
            }

            std::fs::write(&manifest, serde_json::to_string_pretty(&manifest_data)?)?;
        }

        Ok(())
    }
}

impl NeoCliMetricsExporter {
    /// Helper to parse block number from log line
    fn parse_block_number(&self, line: &str) -> Option<u64> {
        // Pattern: "Block #12345" or "#12345 synced"
        let line_lower = line.to_lowercase();

        if let Some(start) = line_lower.find("block #") {
            let after = &line[start + 7..];
            if let Some(_end) = after.find(char::is_alphanumeric) {
                let num_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
                return num_str.parse().ok();
            }
        }

        // Import notices write a bare "#12345" with no "Block" in front of it.
        if let Some(start) = line_lower.find('#') {
            let num_str: String = line[start + 1..]
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if !num_str.is_empty() {
                return num_str.parse().ok();
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_block_number() {
        let exporter = NeoCliMetricsExporter::new(Path::new("/tmp/neo-nexus-neo-cli"));

        assert_eq!(
            exporter.parse_block_number("Block #12345 synced at 15 MB/s"),
            Some(12345)
        );
        assert_eq!(exporter.parse_block_number("#12346 imported"), Some(12346));
        assert_eq!(exporter.parse_block_number("No block info here"), None);
    }
}
