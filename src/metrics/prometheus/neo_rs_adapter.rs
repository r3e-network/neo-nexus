//! Metrics exporter adapter for neo-rs
//!
//! neo-rs uses tokio-console and native tracing-subscriber output.
//! This adapter bridges between tokio-metrics and Prometheus format.

use crate::config::GenerationContext;
use crate::supervisor::model::{
    FatalError, LogEntry, LogParserAdapter, MetricsExporterAdapter as Adapter, PluginMetadata,
    PluginSystemAdapter, SyncProgress,
};
use crate::types::NodeConfig;
use anyhow::Result;
use std::path::Path;

/// Neo-rs metrics adapter - bridges tokio console metrics to Prometheus
#[derive(Debug, Clone)]
pub struct NeoRsMetricsAdapter {
    /// Internal metrics port (tokio-style)
    pub port: u16,
    /// Adapter mode: bridge or raw
    pub mode: AdapterMode,
}

/// Whether to bridge through the tokio-console endpoint instead of reading
/// neo-rs's own Prometheus output. Off until that bridge is built.
const TOKIO_CONSOLE_BRIDGE: bool = false;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AdapterMode {
    /// Bridge through Tokio Console metrics endpoint
    TokioBridge,
    /// Direct Prometheus metrics (if neo-rs exposes them)
    Native,
}

impl NeoRsMetricsAdapter {
    pub fn new(rpc_port: u16) -> Self {
        // neo-rs typically uses Rust native metrics on separate port
        let port = rpc_port;

        // Detect if we should use tokio bridge or native
        let mode = if TOKIO_CONSOLE_BRIDGE {
            AdapterMode::TokioBridge
        } else {
            AdapterMode::Native
        };

        Self { port, mode }
    }

    /// Fetch metrics from neo-rs internal endpoint
    pub fn fetch_metrics(&self) -> Result<String> {
        use reqwest::blocking::Client;
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()?;
        let url = match self.mode {
            AdapterMode::TokioBridge => format!("http://localhost:{}/tokio/console", self.port),
            AdapterMode::Native => format!("http://localhost:{}/metrics", self.port),
        };

        let response = client.get(&url).send()?;
        Ok(response.text()?)
    }

    /// Convert tokio-style metrics to Prometheus exposition format
    pub fn bridge_to_prometheus(&self, raw: String) -> Result<String> {
        let mut prometheus = String::new();

        // Add standard headers
        prometheus.push_str("# HELP neo_rs_runtime Tokio runtime statistics\n");
        prometheus.push_str("# TYPE neo_rs_runtime counter\n");

        // Parse tokio metrics and reformat
        for line in raw.lines() {
            for (name, value) in self.tokio_metrics(line) {
                prometheus.push_str(&format!("neo_rs_{name} {value}\n"));
            }
        }

        Ok(prometheus)
    }

    /// Parse a tokio-style metric line into `thread_key=value` pairs.
    ///
    /// Tokio reports several counters on one line
    /// (`worker idle_count=0 task_queue_depth=42`), so reading only the first
    /// pair would silently drop most of the runtime's own statistics.
    fn tokio_metrics(&self, line: &str) -> Vec<(String, f64)> {
        let mut parts = line.split_whitespace();
        let Some(thread) = parts.next() else {
            return Vec::new();
        };
        let prefix = thread.trim_end_matches('=');
        if prefix.is_empty() {
            return Vec::new();
        }
        parts
            .filter_map(|part| {
                let (key, raw) = part.split_once('=')?;
                let value = raw.parse::<f64>().ok()?;
                Some((format!("{prefix}_{key}"), value))
            })
            .collect()
    }
}

impl Adapter for NeoRsMetricsAdapter {
    fn exporter_package(&self) -> Option<&'static str> {
        match self.mode {
            AdapterMode::TokioBridge => Some("tokio-console"),
            AdapterMode::Native => Some("prometheus-crate"),
        }
    }

    fn metrics_url(&self, _rpc_port: u16) -> Option<String> {
        match self.mode {
            AdapterMode::Native => Some(format!("http://localhost:{}/metrics", self.port)),
            AdapterMode::TokioBridge => None,
        }
    }

    fn generate_config(&self, _node: &NodeConfig) -> Result<Vec<u8>> {
        // Generate TOML config for neo-rs with tokio tracing
        let config = if matches!(self.mode, AdapterMode::TokioBridge) {
            serde_json::json!({
                "tracing": {
                    "enabled": true,
                    "console": {
                        "port": self.port,
                        "addr": format!("127.0.0.1:{}", self.port)
                    }
                },
                "metrics": {
                    "enabled": false, // Use tokio instead
                }
            })
        } else {
            serde_json::json!({
                "tracing": {
                    "enabled": true,
                    "prometheus": {
                        "port": self.port,
                        "path": "/metrics"
                    }
                }
            })
        };

        Ok(serde_json::to_vec_pretty(&config)?)
    }

    fn normalize_metrics(&self, raw: &[u8]) -> Result<String> {
        match self.mode {
            AdapterMode::TokioBridge => {
                // Convert tokio output to Prometheus format
                self.bridge_to_prometheus(String::from_utf8_lossy(raw).to_string())
            }
            AdapterMode::Native => {
                // Already in Prometheus format, just clean it up
                let mut normalized = String::from_utf8_lossy(raw).to_string();

                if !normalized.ends_with('\n') {
                    normalized.push('\n');
                }

                Ok(normalized)
            }
        }
    }
}

impl LogParserAdapter for NeoRsMetricsAdapter {
    fn parse_line(&self, line: &str) -> Option<LogEntry> {
        // neo-rs uses tracing-subscriber format:
        // ERROR consensus::handle: block validation failed target="neox_consensus" time="2024-01-01T00:00:00Z"

        let mut timestamp = 0u64;
        let mut level = "info".to_string();
        let mut source = None;
        let mut message = String::new();

        // Extract level
        if let Some(pos) = line.find(':') {
            let prefix = &line[..pos];
            if prefix.contains("ERROR") {
                level = "error".to_string();
            } else if prefix.contains("WARN") {
                level = "warn".to_string();
            } else if prefix.contains("INFO") {
                level = "info".to_string();
            } else if prefix.contains("DEBUG") {
                level = "debug".to_string();
            }
        }

        // Extract source location (file:line or module path)
        if let Some(start) = line.find('@') {
            if let Some(end) = line[start..].find(' ') {
                source = Some(line[start + 1..start + end].to_string());
            }
        } else if let Some(pkg_start) = line.find("::") {
            // Module path like "consensus::handle"
            let after_pkg = &line[pkg_start + 2..];
            if let Some(colon) = after_pkg.find(':') {
                source = Some(line[pkg_start..pkg_start + 2 + colon].to_string());
            }
        }

        // Extract message (after first ': ')
        if let Some(msg_start) = line.find(": ") {
            message = line[msg_start + 2..].to_string();
        }

        // Extract timestamp if present
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
            metadata: serde_json::json!({"raw_format": "tracing"}),
        })
    }

    fn detect_fatal_errors(&self, log_content: &str) -> Vec<FatalError> {
        let mut errors = Vec::new();

        let fatal_patterns = [
            (" panicked ", "Rust panic occurred - check backtrace"),
            (
                "assertion failed",
                "Debug assertion failed in production code",
            ),
            (
                "unexpected error",
                "Unexpected condition in consensus or RPC",
            ),
            ("database shutdown", "Database layer failure"),
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
        // Look for sync patterns in tracing output
        // Example: "syncing: block=12345/100000 peers=4"

        for line in lines.iter().rev().take(10) {
            if let Some((current, target)) = self.parse_rust_sync_pattern(line) {
                return Some(SyncProgress {
                    current_height: current,
                    target_height: target,
                    sync_percentage: (current as f32 / target as f32) * 100.0,
                    peers_connected: 0, // Would need peer parsing
                });
            }
        }

        None
    }
}

impl PluginSystemAdapter for NeoRsMetricsAdapter {
    fn discover_plugins(&self, _node_dir: &Path) -> Result<Vec<PluginMetadata>> {
        // neo-rs doesn't have runtime plugins - uses Cargo features
        Ok(vec![PluginMetadata {
            id: "features".to_string(),
            name: "Cargo Feature Flags".to_string(),
            version: "N/A".to_string(),
            enabled: false,
        }])
    }

    fn install_plugin(&self, plugin_id: &str, _target_dir: &Path) -> Result<()> {
        // For neo-rs, this means adding a Cargo feature
        anyhow::bail!(
            "neo-rs does not support dynamic plugins; '{}' requires Cargo.toml modification and rebuild",
            plugin_id
        )
    }

    fn toggle_plugin(&self, feature: &str, enabled: bool, ctx: &GenerationContext) -> Result<()> {
        // Modify Cargo.toml to enable/disable feature
        let cargo_toml = super::node_dir_from_context(ctx)?.join("Cargo.toml");

        if cargo_toml.exists() {
            let mut content = std::fs::read_to_string(&cargo_toml)?;

            // Simple text manipulation - would ideally parse TOML properly
            let _feature_line = format!("{} = [\"{}\"];", "default", feature);

            if enabled && !content.contains(feature) {
                // Add feature to default list
                content = content.replace(
                    r#"default = ["std"]"#,
                    &format!(r#"default = ["std", "{}"]"#, feature),
                );
            } else if !enabled {
                // Remove feature from default list
                content = content.replace(&format!(r#"["std", "{}"]"#, feature), r#"["std"]"#);
            }

            std::fs::write(&cargo_toml, &content)?;

            return Ok(());
        }

        anyhow::bail!("Cargo.toml not found at {:?}", cargo_toml)
    }
}

impl NeoRsMetricsAdapter {
    /// Parse ISO8601 timestamp from tracing output
    fn parse_timestamp(&self, ts: &str) -> u64 {
        // Same parser as neo-go (common format)
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

    /// Parse sync pattern from Rust-style log line
    fn parse_rust_sync_pattern(&self, line: &str) -> Option<(u64, u64)> {
        // Pattern: "block=X/Y" or "sync X/Y"

        if let Some(block_pos) = line.find("block=") {
            let after = &line[block_pos + 6..];
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
    fn test_parse_tracing_line() {
        let adapter = NeoRsMetricsAdapter::new(20332);

        let line = r#"ERROR consensus::handle: block validation failed @src/consensus/handle.rs:42 time="2024-01-01T12:00:00Z""#;
        let entry = adapter.parse_line(line).unwrap();

        assert_eq!(entry.level, "error");
        assert!(entry.source.as_ref().unwrap().contains("consensus"));
        assert!(entry.message.contains("validation failed"));
    }

    #[test]
    fn test_bridge_tokio_metrics() {
        let adapter = NeoRsMetricsAdapter::new(20332);

        let tokio_output = "worker idle_count=0 task_queue_depth=42";
        let result = adapter
            .bridge_to_prometheus(tokio_output.to_string())
            .unwrap();

        assert!(result.contains("neo_rs_worker"));
        assert!(result.contains("task_queue_depth"));
    }
}
