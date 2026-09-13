//! Metrics exporter adapter for Neo X Reth
//!
//! Neo X Reth uses reth's native Rust-based metrics system.
//! This adapter bridges Reth metrics to Prometheus format.

use crate::config::GenerationContext;
use crate::supervisor::model::{
    FatalError, LogEntry, LogParserAdapter, MetricsExporterAdapter as Adapter, PluginMetadata,
    PluginSystemAdapter, SyncProgress,
};
use crate::types::NodeConfig;
use anyhow::Result;
use std::path::Path;

/// Neo X Reth metrics adapter - extends Reth's native metrics
#[derive(Debug, Clone)]
pub struct NeoXRethMetricsAdapter {
    /// HTTP RPC port (Reth default)
    pub http_port: u16,
    /// Metrics port (separate from RPC)
    pub metrics_port: u16,
}

impl NeoXRethMetricsAdapter {
    pub fn new(rpc_port: u16) -> Self {
        let http_port = rpc_port;
        let metrics_port = 9091; // Default Reth metrics port

        Self {
            http_port,
            metrics_port,
        }
    }

    /// Get metrics endpoint URL
    pub fn metrics_url(&self) -> String {
        format!("http://localhost:{}{}", self.metrics_port, "/metrics")
    }

    /// Fetch metrics from Reth's Prometheus endpoint
    pub fn fetch_metrics(&self) -> Result<String> {
        use reqwest::blocking::Client;
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()?;

        let response = client.get(self.metrics_url()).send()?;
        Ok(response.text()?)
    }

    /// Add Neo X-specific labels to Reth metrics
    pub fn add_neo_labels(&self, metrics: String, node_id: &str) -> String {
        let mut labeled = String::new();

        for line in metrics.lines() {
            if line.starts_with('#') {
                labeled.push_str(line);
                labeled.push('\n');
                continue;
            }

            if line.is_empty() || line.starts_with('#') {
                labeled.push_str(line);
                labeled.push('\n');
                continue;
            }

            // Add Neo X chain identifier
            if !line.contains('{') {
                labeled.push_str(line);
                labeled.push_str("{neo_node_id=\"");
                labeled.push_str(node_id);
                labeled.push_str("\",neo_chain=\"neox\"} ");
            } else {
                let Some(insert_pos) = line.find('{').map(|position| position + 1) else {
                    labeled.push_str(line);
                    labeled.push('\n');
                    continue;
                };
                labeled.push_str(&line[..insert_pos]);
                labeled.push_str("neo_node_id=\"");
                labeled.push_str(node_id);
                labeled.push_str("\",neo_chain=\"neox\",");
                labeled.push_str(&line[insert_pos..]);
            }
        }

        labeled
    }
}

impl Adapter for NeoXRethMetricsAdapter {
    fn exporter_package(&self) -> Option<&'static str> {
        None // Built into Reth binary
    }

    fn metrics_url(&self, _rpc_port: u16) -> Option<String> {
        Some(format!("http://localhost:{}/metrics", self.metrics_port))
    }

    fn generate_config(&self, _node: &NodeConfig) -> Result<Vec<u8>> {
        // Generate JSON config for Reth with metrics enabled
        let config = serde_json::json!({
            "rpc": {
                "http": true,
                "http_addr": format!("127.0.0.1:{}", self.http_port),
                "http_port": self.http_port
            },
            "metrics": {
                "enabled": true,
                "addr": format!("127.0.0.1:{}", self.metrics_port),
                "port": self.metrics_port
            },
            "consensus": {
                "debug": false
            }
        });

        Ok(serde_json::to_vec_pretty(&config)?)
    }

    fn normalize_metrics(&self, raw: &[u8]) -> Result<String> {
        // Reth exports standard Prometheus format
        let mut normalized = String::from_utf8_lossy(raw).to_string();

        if !normalized.ends_with('\n') {
            normalized.push('\n');
        }

        Ok(normalized)
    }
}

impl LogParserAdapter for NeoXRethMetricsAdapter {
    fn parse_line(&self, line: &str) -> Option<LogEntry> {
        // Reth uses tracing-subscriber like neo-rs:
        // ERROR reth_consensus: block validation failed target="reth" time="2024-01-01T00:00:00Z"

        let mut timestamp = 0u64;
        let mut level = "info".to_string();
        let mut source = None;
        let mut message = String::new();

        // Extract level
        if line.starts_with("ERROR") {
            level = "error".to_string();
        } else if line.starts_with("WARN") {
            level = "warn".to_string();
        } else if line.starts_with("INFO") {
            level = "info".to_string();
        } else if line.starts_with("DEBUG") || line.starts_with("TRACE") {
            level = "debug".to_string();
        }

        // Extract module prefix (e.g., "reth_consensus:")
        if let Some(colon_pos) = line.find(':') {
            if colon_pos > 0 {
                source = Some(line[..colon_pos].trim().to_string());

                // Message is after ": "
                if colon_pos + 2 < line.len() {
                    message = line[colon_pos + 2..].to_string();
                }
            }
        }

        // Extract timestamp
        if let Some(ts_pos) = line.find("time=\"") {
            let ts_end = line[ts_pos..].find('"').map(|p| ts_pos + p + 1)?;
            let ts_str = &line[ts_pos + 6..ts_end];
            timestamp = self.parse_timestamp(ts_str);
        }

        Some(LogEntry {
            timestamp,
            level,
            message,
            source,
            metadata: serde_json::json!({"adapter": "neox_reth"}),
        })
    }

    fn detect_fatal_errors(&self, log_content: &str) -> Vec<FatalError> {
        let mut errors = Vec::new();

        let fatal_patterns = [
            ("thread 'main' panicked", "Rust main thread panic"),
            ("database error:", "MDBX database failure"),
            ("snapshot load failed", "Snapshot initialization error"),
            ("forkchoice update failed", "Consensus rule violation"),
            ("no peers connected", "Network partition detected"),
            ("state pruning failed", "Pruner subsystem crashed"),
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
        // Reth sync patterns:
        // "completed downloading block hash=abc number=12345/100000"
        // "imported block number=12345 state_size=1.2GB"

        for line in lines.iter().rev().take(20) {
            if let Some((current, target)) = self.parse_reth_sync(line) {
                return Some(SyncProgress {
                    current_height: current,
                    target_height: target,
                    sync_percentage: (current as f32 / target as f32) * 100.0,
                    peers_connected: 0, // Would need network stats parsing
                });
            }
        }

        None
    }
}

impl PluginSystemAdapter for NeoXRethMetricsAdapter {
    fn discover_plugins(&self, node_dir: &Path) -> Result<Vec<PluginMetadata>> {
        // Reth uses extensions via Cargo features and dynamic loading
        let ext_dir = node_dir.join("extensions");
        if !ext_dir.exists() {
            return Ok(Vec::new());
        }

        let mut plugins = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&ext_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with("lib") || name.ends_with(".dll") {
                        plugins.push(PluginMetadata {
                            id: name.to_string(),
                            name: name.to_string(),
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
        // For Reth, this means adding an extension to build
        let ext_dir = target_dir.join("extensions");
        std::fs::create_dir_all(&ext_dir)?;

        // Create placeholder manifest
        let manifest = ext_dir.join(format!("{}.manifest.json", plugin_id));
        std::fs::write(
            &manifest,
            serde_json::to_string_pretty(&serde_json::json!({
                "id": plugin_id,
                "version": "1.0.0",
                "type": "reth-extension",
                "enabled": true,
                "requires_build": true
            }))?,
        )?;

        Ok(())
    }

    fn toggle_plugin(&self, plugin_id: &str, enabled: bool, ctx: &GenerationContext) -> Result<()> {
        // Update extension manifest
        let ext_dir = super::node_dir_from_context(ctx)?.join("extensions");
        let manifest = ext_dir.join(format!("{}.manifest.json", plugin_id));

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

impl NeoXRethMetricsAdapter {
    /// Parse ISO8601 timestamp
    fn parse_timestamp(&self, ts: &str) -> u64 {
        let cleaned = ts.trim_end_matches('Z').trim_end_matches('+');
        let parts: Vec<&str> = cleaned.split('-').collect();

        if parts.len() >= 3 {
            if let (Ok(year), Ok(month), Some(day)) = (
                parts[0].parse::<i64>(),
                parts[1].parse::<i32>(),
                parts[2]
                    .split(' ')
                    .next()
                    .and_then(|d| d.parse::<i32>().ok()),
            ) {
                return ((year - 1970) * 31536000
                    + (month as i64 - 1) * 2592000
                    + (day as i64 - 1) * 86400) as u64;
            }
        }

        0
    }

    /// Parse sync progress from Reth log line
    fn parse_reth_sync(&self, line: &str) -> Option<(u64, u64)> {
        // Pattern: "number=X/Y" or "progress=X/Y"

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

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_reth_log() {
        let adapter = NeoXRethMetricsAdapter::new(3030);

        let line = "ERROR reth_consensus: block validation failed @src/consensus/mod.rs:123";
        let entry = adapter.parse_line(line).unwrap();

        assert_eq!(entry.level, "error");
        assert!(entry.source.as_ref().unwrap().contains("reth_consensus"));
        assert!(entry.message.contains("validation failed"));
    }
}
