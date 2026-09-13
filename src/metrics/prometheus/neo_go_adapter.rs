//! Metrics exporter adapter for neo-go
//!
//! neo-go has built-in Prometheus metrics endpoint at `/metrics` path.
//! This adapter exposes and normalizes those metrics with Neo-specific labels.

use crate::config::GenerationContext;
use crate::supervisor::model::{
    FatalError, LogEntry, LogParserAdapter, MetricsExporterAdapter as Adapter, PluginMetadata,
    PluginSystemAdapter, SyncProgress,
};
use crate::types::NodeConfig;
use anyhow::Result;
use std::path::Path;

/// Neo-go metrics adapter - uses built-in prometheus endpoint
#[derive(Debug, Clone)]
pub struct NeoGoMetricsAdapter {
    /// RPC port where metrics are exposed (RPC port + /metrics suffix)
    pub port: u16,
    /// Metrics path
    pub path: &'static str,
}

impl NeoGoMetricsAdapter {
    pub fn new(rpc_port: u16) -> Self {
        // neo-go typically exposes metrics on same port as RPC
        let port = rpc_port;

        Self {
            port,
            path: "/metrics",
        }
    }

    /// Fetch metrics from neo-go's built-in endpoint
    pub fn fetch_metrics(&self) -> Result<String> {
        use reqwest::blocking::Client;
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()?;
        let url = format!("http://localhost:{}/metrics", self.port);

        let response = client.get(&url).send()?;
        Ok(response.text()?)
    }

    /// Add Neo-specific labels to metric families
    pub fn add_labels(&self, metrics: String, node_id: &str) -> String {
        let mut labeled = String::new();

        for line in metrics.lines() {
            // Skip help and type comments
            if line.starts_with("#") {
                labeled.push_str(line);
                labeled.push('\n');
                continue;
            }

            // Add neo_node_id label to counter/gauge metrics
            if !line.is_empty() && !line.contains(" ") {
                labeled.push_str(line);
                labeled.push_str("{neo_node_id=\"");
                labeled.push_str(node_id);
                labeled.push_str("\"} ");
            } else {
                labeled.push_str(line);
                labeled.push('\n');
            }
        }

        labeled
    }
}

impl Adapter for NeoGoMetricsAdapter {
    fn exporter_package(&self) -> Option<&'static str> {
        Some("prometheus-exporter") // Built into neo-go binary
    }

    fn metrics_url(&self, _rpc_port: u16) -> Option<String> {
        Some(format!("http://localhost:{}{}", self.port, self.path))
    }

    fn generate_config(&self, _node: &NodeConfig) -> Result<Vec<u8>> {
        // Generate YAML config for neo-go metrics
        let config = serde_json::json!({
            "prometheus": {
                "enabled": true,
                "address": format!(":{}", self.port),
                "path": self.path,
                "namespace": "neo_go"
            }
        });

        Ok(serde_yaml::to_string(&config)?.into_bytes())
    }

    fn normalize_metrics(&self, raw: &[u8]) -> Result<String> {
        // neo-go exports in standard Prometheus format
        // Just ensure trailing newline and consistent formatting
        let mut normalized = String::from_utf8_lossy(raw).to_string();

        if !normalized.ends_with('\n') {
            normalized.push('\n');
        }

        Ok(normalized)
    }
}

/// Split a log line on whitespace, leaving `msg="a message"` in one piece.
///
/// neo-go quotes its message field, and a plain whitespace split reported
/// every multi-word message as its first word only.
fn split_outside_quotes(line: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = None;
    let mut in_quote = false;
    for (index, character) in line.char_indices() {
        if character == '"' {
            in_quote = !in_quote;
            start.get_or_insert(index);
        } else if character.is_whitespace() && !in_quote {
            if let Some(from) = start.take() {
                parts.push(&line[from..index]);
            }
        } else {
            start.get_or_insert(index);
        }
    }
    if let Some(from) = start {
        parts.push(&line[from..]);
    }
    parts
}

impl LogParserAdapter for NeoGoMetricsAdapter {
    fn parse_line(&self, line: &str) -> Option<LogEntry> {
        // neo-go logs format: time="2024-01-01T00:00:00Z" level=info pkg=consensus msg="message"
        let parts = split_outside_quotes(line);

        if parts.is_empty() {
            return None;
        }

        let mut timestamp = 0u64;
        let mut level = "info".to_string();
        let mut source = None;
        let mut message = String::new();
        let mut metadata = serde_json::Map::new();

        // Parse key=value pairs
        for part in parts {
            if let Some(eq_pos) = part.find('=') {
                let key = &part[..eq_pos];
                let value = &part[eq_pos + 1..];

                match key {
                    "time" => {
                        // Parse ISO timestamp to Unix epoch
                        timestamp = self.parse_timestamp(value.trim_matches('"'));
                    }
                    "level" => {
                        level = value.trim_matches('"').to_string();
                    }
                    "pkg" => {
                        source = Some(value.trim_matches('"').to_string());
                    }
                    "msg" => {
                        message = value.trim_matches('"').to_string();
                    }
                    _ => {
                        metadata
                            .insert(key.to_string(), serde_json::json!(value.trim_matches('"')));
                    }
                }
            }
        }

        // If we couldn't extract message from kv pairs, use entire line
        if message.is_empty() {
            message = line.to_string();
        }

        Some(LogEntry {
            timestamp,
            level,
            message,
            source,
            metadata: serde_json::Value::Object(metadata),
        })
    }

    fn detect_fatal_errors(&self, log_content: &str) -> Vec<FatalError> {
        let mut errors = Vec::new();

        let fatal_patterns = [
            ("panic", "Neo-go encountered a panic - check stack trace"),
            ("fatal error", "Fatal error in go runtime or neo-go"),
            (
                "consensus failure",
                "Consensus module detected critical issue",
            ),
            ("db error", "Database/backend storage error"),
            ("failed to start", "Node failed to initialize properly"),
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
        // Look for sync patterns like: block 12345/100000 synced
        for line in lines.iter().rev().take(20) {
            if let Some((current, target)) = self.parse_sync_range(line) {
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

impl PluginSystemAdapter for NeoGoMetricsAdapter {
    fn discover_plugins(&self, _node_dir: &Path) -> Result<Vec<PluginMetadata>> {
        // neo-go doesn't use traditional plugins - uses Go modules
        // Return empty list but indicate this is module-based
        Ok(vec![PluginMetadata {
            id: "modules".to_string(),
            name: "Go Modules".to_string(),
            version: "N/A".to_string(),
            enabled: false,
        }])
    }

    fn install_plugin(&self, plugin_id: &str, _target_dir: &Path) -> Result<()> {
        // For neo-go, "installing" means adding go.mod dependency
        // This requires recompilation - not hot-loading
        anyhow::bail!(
            "neo-go does not support hot-plugin loading; '{}' requires go.mod update and rebuild",
            plugin_id
        )
    }

    fn toggle_plugin(
        &self,
        plugin_id: &str,
        _enabled: bool,
        _ctx: &GenerationContext,
    ) -> Result<()> {
        anyhow::bail!(
            "neo-go module '{}' cannot be toggled dynamically; requires code change and rebuild",
            plugin_id
        )
    }
}

impl NeoGoMetricsAdapter {
    /// Parse ISO8601 timestamp to Unix epoch
    fn parse_timestamp(&self, ts: &str) -> u64 {
        // Simple parser for common neo-go format
        // "2024-01-01T00:00:00Z" or similar

        // Remove timezone indicators and parse manually
        let cleaned = ts.replace(['Z', '+'], "");
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
                // Simplified epoch calculation (not exact but sufficient for logging)
                return ((year - 1970) * 31536000
                    + (month as i64 - 1) * 2592000
                    + (day as i64 - 1) * 86400) as u64;
            }
        }

        0
    }

    /// Parse sync progress range "X/Y" from log line
    fn parse_sync_range(&self, line: &str) -> Option<(u64, u64)> {
        if let Some(pos) = line.find("block ") {
            let after = &line[pos + 6..];
            if let Some(slash_pos) = after.find('/') {
                let current = after[..slash_pos].parse::<u64>().ok()?;
                let target = after[slash_pos + 1..]
                    .split(|c: char| !c.is_ascii_digit())
                    .next()?
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
    fn test_parse_sync_range() {
        let adapter = NeoGoMetricsAdapter::new(20332);

        assert_eq!(
            adapter.parse_sync_range("block 12345/100000 synced"),
            Some((12345, 100000))
        );
        assert_eq!(adapter.parse_sync_range("#100/1000 imported"), None); // Different format
        assert_eq!(adapter.parse_sync_range("No sync info"), None);
    }

    #[test]
    fn test_parse_log_line() {
        let adapter = NeoGoMetricsAdapter::new(20332);

        let line = r#"time="2024-01-01T12:00:00Z" level=info pkg=consensus msg="block processed""#;
        let entry = adapter.parse_line(line).unwrap();

        assert_eq!(entry.level, "info");
        assert_eq!(entry.source, Some("consensus".to_string()));
        assert_eq!(entry.message, "block processed");
    }
}
