//! Metrics exporter adapter for Neo X Geth
//!
//! Neo X Geth inherits Ethereum geth's built-in Prometheus metrics.
//! This adapter preserves EVM-compatible metric names while adding Neo X labels.

use crate::config::GenerationContext;
use crate::supervisor::model::{
    FatalError, LogEntry, LogParserAdapter, MetricsExporterAdapter as Adapter, PluginMetadata,
    PluginSystemAdapter, SyncProgress,
};
use crate::types::NodeConfig;
use anyhow::Result;
use std::path::Path;

/// Neo X Geth metrics adapter - extends geth Prometheus endpoint
#[derive(Debug, Clone)]
pub struct NeoXGethMetricsAdapter {
    /// HTTP RPC port where metrics are exposed (same as HTTP API)
    pub http_port: u16,
    /// Metrics endpoint path
    pub metrics_path: &'static str,
}

impl NeoXGethMetricsAdapter {
    pub fn new(rpc_port: u16) -> Self {
        let http_port = rpc_port;

        Self {
            http_port,
            metrics_path: "/metrics",
        }
    }

    /// Get full metrics URL for Neo X Geth
    pub fn metrics_url(&self) -> String {
        format!("http://localhost:{}{}", self.http_port, self.metrics_path)
    }

    /// Fetch metrics from geth's built-in Prometheus endpoint
    pub fn fetch_metrics(&self) -> Result<String> {
        use reqwest::blocking::Client;
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()?;

        let response = client.get(self.metrics_url()).send()?;
        Ok(response.text()?)
    }

    /// Add Neo X chain labels to EVM metrics
    pub fn add_chain_labels(&self, metrics: String, node_id: &str) -> String {
        let mut labeled = String::new();
        let chain_label = "neo_x";

        for line in metrics.lines() {
            // Skip help comments
            if line.starts_with("# HELP") || line.starts_with("# TYPE") {
                labeled.push_str(line);
                labeled.push('\n');
                continue;
            }

            // Check if this is a real metric line
            if line.is_empty() || line.starts_with('#') {
                labeled.push_str(line);
                labeled.push('\n');
                continue;
            }

            // Add neo_node_id and neo_chain labels
            if !line.contains("{") && !line.contains("}") {
                // No existing labels - add them
                labeled.push_str(line);
                labeled.push_str("{neo_node_id=\"");
                labeled.push_str(node_id);
                labeled.push_str("\",neo_chain=\"");
                labeled.push_str(chain_label);
                labeled.push_str("\"} ");
            } else {
                // Has existing labels - insert after opening brace
                let Some(insert_pos) = line.find('{').map(|position| position + 1) else {
                    labeled.push_str(line);
                    labeled.push('\n');
                    continue;
                };
                labeled.push_str(&line[..insert_pos]);
                labeled.push_str("neo_node_id=\"");
                labeled.push_str(node_id);
                labeled.push_str("\",neo_chain=\"");
                labeled.push_str(chain_label);
                labeled.push_str("\",");
                labeled.push_str(&line[insert_pos..]);
            }
        }

        labeled
    }
}

impl Adapter for NeoXGethMetricsAdapter {
    fn exporter_package(&self) -> Option<&'static str> {
        None // Built into geth binary
    }

    fn metrics_url(&self, _rpc_port: u16) -> Option<String> {
        Some(format!(
            "http://localhost:{}{}",
            self.http_port, self.metrics_path
        ))
    }

    fn generate_config(&self, _node: &NodeConfig) -> Result<Vec<u8>> {
        // Generate JSON config for geth with metrics enabled
        let config = serde_json::json!({
            "http": true,
            "http.port": self.http_port,
            "http.vhosts": ["*"],
            "gcmodes": {
                "pruneSize": 4096
            },
            "metrics": {
                "expansion": true,
                "account": true,
                "transaction": true,
                "vm": true
            }
        });

        Ok(serde_json::to_vec_pretty(&config)?)
    }

    fn normalize_metrics(&self, raw: &[u8]) -> Result<String> {
        // Geth exports standard Prometheus format
        // Just ensure proper line endings
        let mut normalized = String::from_utf8_lossy(raw).to_string();

        if !normalized.ends_with('\n') {
            normalized.push('\n');
        }

        Ok(normalized)
    }
}

impl LogParserAdapter for NeoXGethMetricsAdapter {
    fn parse_line(&self, line: &str) -> Option<LogEntry> {
        // Geth logs format varies by version but typically:
        // "01-01 12:00:00 [TRACING] imported 123 blocks"
        // or Go-style: "level=info module=ethereum time="2024-01-01T00:00:00Z""

        let mut timestamp = 0u64;
        let mut level = "info".to_string();
        let mut source = None;
        let mut message = line.to_string();

        // Detect level
        let lower = line.to_lowercase();
        if lower.contains("error") || lower.contains("fail") {
            level = "error".to_string();
        } else if lower.contains("warn") {
            level = "warn".to_string();
        } else if lower.contains("trace") {
            level = "debug".to_string();
        }

        // Try Go-style key=value parsing
        if lower.contains("level=") {
            if let Some(msg_start) = line.find("msg=\"") {
                let after_msg = &line[msg_start + 5..];
                if let Some(end) = after_msg.find('"') {
                    message = after_msg[..end].to_string();
                }
            }
        }

        // Try bracketed timestamp format "[MM-DD HH:MM:SS]"
        if let Some(bracket_start) = line.find('[') {
            if let Some(bracket_end) = line[bracket_start..].find(']') {
                let ts_part = &line[bracket_start + 1..bracket_start + bracket_end];
                // Parse "MM-DD HH:MM:SS"
                let parts: Vec<&str> = ts_part.split_whitespace().collect();
                if parts.len() == 2 {
                    timestamp = self.parse_date_time(parts[0], parts[1]);
                }
            }
        }

        // Extract source/module info
        if let Some(module_pos) = line.find("module=") {
            source = Some(
                line[module_pos + 7..]
                    .split(' ')
                    .next()
                    .unwrap_or("")
                    .trim_matches('"')
                    .to_string(),
            );
        }

        Some(LogEntry {
            timestamp,
            level,
            message,
            source,
            metadata: serde_json::json!({"adapter": "neox_geth"}),
        })
    }

    fn detect_fatal_errors(&self, log_content: &str) -> Vec<FatalError> {
        let mut errors = Vec::new();

        let fatal_patterns = [
            ("panic", "Geth runtime panic - check goroutine dump"),
            (
                "critical error",
                "Critical failure in consensus or networking",
            ),
            (
                "database corruption",
                "Pebble/MDBX database integrity issue",
            ),
            ("snapshots unavailable", "Snapshot system failed to load"),
            ("chain reorg", "Major chain reorganization detected"),
            ("disk full", "Storage subsystem exhausted"),
        ];

        for (pattern, suggestion) in &fatal_patterns {
            if log_content.to_lowercase().contains(pattern) {
                errors.push(FatalError {
                    line_number: 0,
                    pattern: (*pattern).to_string(),
                    suggestion: suggestion.to_string(),
                });
            }
        }

        errors
    }

    fn extract_sync_progress(&self, lines: &[&str]) -> Option<SyncProgress> {
        // Geth sync patterns:
        // "imported block number=12345 hash=abc... elapsed=1.2s"
        // "syncing: downloader progress=12345/100000"

        for line in lines.iter().rev().take(20) {
            if let Some((current, target)) = self.parse_geth_sync(line) {
                return Some(SyncProgress {
                    current_height: current,
                    target_height: target,
                    sync_percentage: (current as f32 / target as f32) * 100.0,
                    peers_connected: 0, // Would need peer table parsing
                });
            }
        }

        None
    }
}

impl PluginSystemAdapter for NeoXGethMetricsAdapter {
    fn discover_plugins(&self, node_dir: &Path) -> Result<Vec<PluginMetadata>> {
        // Geth uses plugin system via external go plugins
        let plugin_dir = node_dir.join("plugins");
        if !plugin_dir.exists() {
            return Ok(Vec::new());
        }

        let mut plugins = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&plugin_dir) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext.to_string_lossy().starts_with("so")
                        || ext.to_string_lossy().starts_with("dll")
                    {
                        let name = entry
                            .path()
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        plugins.push(PluginMetadata {
                            id: name.clone(),
                            name: name.clone(),
                            version: "N/A".to_string(),
                            enabled: true,
                        });
                    }
                }
            }
        }

        Ok(plugins)
    }

    fn install_plugin(&self, plugin_id: &str, target_dir: &Path) -> Result<()> {
        // Copy plugin to geth's plugins directory
        let plugin_dir = target_dir.join("plugins");
        std::fs::create_dir_all(&plugin_dir)?;

        // In real usage, would download plugin binary and verify signature
        let manifest = plugin_dir.join(format!("{}.manifest.json", plugin_id));
        std::fs::write(
            &manifest,
            serde_json::to_string_pretty(&serde_json::json!({
                "id": plugin_id,
                "version": "1.0.0",
                "enabled": true,
                "type": "go-plugin"
            }))?,
        )?;

        Ok(())
    }

    fn toggle_plugin(&self, plugin_id: &str, enabled: bool, ctx: &GenerationContext) -> Result<()> {
        // Update plugin manifest
        let plugin_dir = super::node_dir_from_context(ctx)?.join("plugins");
        let manifest = plugin_dir.join(format!("{}.manifest.json", plugin_id));

        if manifest.exists() {
            let mut data: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(&manifest)?)?;

            if let Some(obj) = data.as_object_mut() {
                obj.insert("enabled".to_string(), serde_json::json!(enabled));
            }

            std::fs::write(&manifest, serde_json::to_string_pretty(&data)?)?;
        }

        Ok(())
    }
}

impl NeoXGethMetricsAdapter {
    /// Parse date-time string "MM-DD HH:MM:SS" to epoch
    fn parse_date_time(&self, date: &str, time: &str) -> u64 {
        // Very simplified parser - assumes current year
        let parts: Vec<&str> = date.split('-').collect();
        let time_parts: Vec<&str> = time.split(':').collect();

        if parts.len() == 2 && time_parts.len() == 3 {
            if let (Ok(month), Ok(day), Ok(hour), Ok(min), Ok(sec)) = (
                parts[0].parse::<i32>(),
                parts[1].parse::<i32>(),
                time_parts[0].parse::<i32>(),
                time_parts[1].parse::<i32>(),
                time_parts[2].parse::<i32>(),
            ) {
                // Use a fixed reference year (2024) for approximation
                let year = 2024;
                if let Ok(unix_epoch) = chrono::DateTime::parse_from_rfc3339(&format!(
                    "{}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                    year, month, day, hour, min, sec
                )) {
                    return unix_epoch.timestamp() as u64;
                }
            }
        }

        0
    }

    /// Parse sync progress from geth log line
    fn parse_geth_sync(&self, line: &str) -> Option<(u64, u64)> {
        // Pattern 1: "number=12345/100000"
        if let Some(num_start) = line.find("number=") {
            let after = &line[num_start + 7..];
            if let Some(slash) = after.find('/') {
                let current = after[..slash].parse::<u64>().ok()?;
                let target = after[slash + 1..]
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse::<u64>()
                    .ok()?;
                return Some((current, target));
            }
        }

        // Pattern 2: "progress=12345/100000"
        if let Some(prog_start) = line.find("progress=") {
            let after = &line[prog_start + 9..];
            if let Some(slash) = after.find('/') {
                let current = after[..slash].parse::<u64>().ok()?;
                let target = after[slash + 1..]
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse::<u64>()
                    .ok()?;
                return Some((current, target));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_geth_sync_pattern() {
        let adapter = NeoXGethMetricsAdapter::new(8545);

        assert_eq!(
            adapter.parse_geth_sync("imported block number=12345/100000"),
            Some((12345, 100000))
        );
        assert_eq!(
            adapter.parse_geth_sync("downloader progress=500/1000"),
            Some((500, 1000))
        );
        assert_eq!(adapter.parse_geth_sync("No sync info"), None);
    }
}
