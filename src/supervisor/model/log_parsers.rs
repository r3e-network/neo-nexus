//! Log parser adapters: one per node type, turning raw log lines into the
//! structured entries, fatal-error reports and sync progress the workbench
//! renders.
//!
//! The helpers below are deliberately private: every one of them exists to
//! serve the parsers in this file, and none of them is part of the adapter
//! surface.

use serde::{Deserialize, Serialize};

mod helpers;
use helpers::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: u64,
    pub level: String,
    pub message: String,
    pub source: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FatalError {
    pub line_number: u32,
    pub pattern: String,
    pub suggestion: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncProgress {
    pub current_height: u64,
    pub target_height: u64,
    pub sync_percentage: f32,
    pub peers_connected: u32,
}

/// Trait for parsing and analyzing node log files across different runtime types.
pub trait LogParserAdapter: std::fmt::Debug + Send + Sync {
    /// Parse a single log line into structured components.
    /// Returns None if the line doesn't match expected format.
    fn parse_line(&self, line: &str) -> Option<LogEntry>;

    /// Scan entire log content for fatal errors that require operator intervention.
    /// Returns vector of potential issues with line numbers and suggested fixes.
    fn detect_fatal_errors(&self, log_content: &str) -> Vec<FatalError>;

    /// Extract synchronization progress from recent log lines in chronological order.
    /// Returns Some only when both current and target heights are known.
    fn extract_sync_progress(&self, lines: &[&str]) -> Option<SyncProgress>;
}

#[derive(Debug, Clone)]
pub struct NeoCliLogParser;

impl LogParserAdapter for NeoCliLogParser {
    fn parse_line(&self, line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(4, ']').collect();
        if parts.len() < 2 {
            return None;
        }

        let timestamp_str = parts[0].trim_start_matches('[');
        let level_part = parts[1].trim_start_matches('[');
        let component = parts.get(2).map(|s| s.trim().to_string());
        let message = parts
            .get(3)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        let timestamp = timestamp_str.parse::<u64>().ok()?;
        let level = level_part.to_string().to_uppercase();

        Some(LogEntry {
            timestamp,
            level,
            message,
            source: component,
            metadata: serde_json::json!({"parser": "neo-cli", "format": "bracketed"}),
        })
    }

    fn detect_fatal_errors(&self, log_content: &str) -> Vec<FatalError> {
        let mut errors = Vec::new();
        for (line_num, line) in log_content.lines().enumerate() {
            if line.to_uppercase().contains("FATAL") || line.to_uppercase().contains("PANIC") {
                errors.push(FatalError {
                    line_number: (line_num + 1) as u32,
                    pattern: "FATAL/PANIC".to_string(),
                    suggestion: "Check database integrity and restart with recovery mode enabled"
                        .to_string(),
                });
            }
        }
        errors
    }

    fn extract_sync_progress(&self, lines: &[&str]) -> Option<SyncProgress> {
        for line in lines.iter().rev().take(10) {
            if let Some(current) = extract_bracket_number(line, "height") {
                if let Some(target) = extract_bracket_number(line, "of") {
                    if let Some(progress) =
                        measured_sync_progress(current, target, extract_peer_count(line))
                    {
                        return Some(progress);
                    }
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct NeoGoLogParser;

impl LogParserAdapter for NeoGoLogParser {
    fn parse_line(&self, line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.len() < 3 {
            return None;
        }

        let ts_str = parts[0];
        let level = parts[1].to_uppercase();
        let rest = parts[2];

        let module = rest.split(':').next().unwrap_or("").trim().to_string();
        let message = rest
            .split_once(':')
            .map(|(_, m)| m.trim())
            .unwrap_or("")
            .to_string();

        let timestamp = ts_parse_timestamp(ts_str)?;

        Some(LogEntry {
            timestamp,
            level,
            message,
            source: Some(module),
            metadata: serde_json::json!({"parser": "neo-go", "format": "golang"}),
        })
    }

    fn detect_fatal_errors(&self, log_content: &str) -> Vec<FatalError> {
        let mut errors = Vec::new();
        for (line_num, line) in log_content.lines().enumerate() {
            if line.to_lowercase().contains("fatal") || line.to_lowercase().contains("panic") {
                errors.push(FatalError {
                    line_number: (line_num + 1) as u32,
                    pattern: "FATAL/PANIC".to_string(),
                    suggestion: "Verify config.yml correctness and check port availability"
                        .to_string(),
                });
            }
        }
        errors
    }

    fn extract_sync_progress(&self, lines: &[&str]) -> Option<SyncProgress> {
        for line in lines.iter().rev().take(10) {
            if let (Some(current), Some(target)) = (
                extract_neogo_height(line),
                extract_bracket_number(line, "targetheight="),
            ) {
                if let Some(progress) =
                    measured_sync_progress(current, target, extract_peer_count(line))
                {
                    return Some(progress);
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct NeoRsLogParser;

impl LogParserAdapter for NeoRsLogParser {
    fn parse_line(&self, line: &str) -> Option<LogEntry> {
        let trimmed = line.trim();
        if !trimmed.starts_with('[') {
            return None;
        }

        let parts: Vec<&str> = trimmed.splitn(3, "]").collect();
        if parts.len() < 2 {
            return None;
        }

        let timestamp = parts[0].trim_start_matches('[');
        let level_and_msg = parts[1].trim();

        let (level, message) = if let Some(idx) = level_and_msg.find(':') {
            let level = level_and_msg[..idx].trim().to_uppercase();
            let msg = level_and_msg[idx + 1..].trim();
            (level, msg)
        } else {
            ("INFO".to_string(), level_and_msg)
        };

        let timestamp_u64 = timestamp.parse::<u64>().ok()?;

        Some(LogEntry {
            timestamp: timestamp_u64,
            level,
            message: message.to_string(),
            source: None,
            metadata: serde_json::json!({"parser": "neo-rs", "format": "tokio"}),
        })
    }

    fn detect_fatal_errors(&self, log_content: &str) -> Vec<FatalError> {
        let mut errors = Vec::new();
        for (line_num, line) in log_content.lines().enumerate() {
            if contains_any_case(line, &["panicked at", "panic:", "fatal:"]) {
                errors.push(FatalError {
                    line_number: (line_num + 1) as u32,
                    pattern: "PANIC/FATAL".to_string(),
                    suggestion: "Check rust panic backtrace and runtime dependencies".to_string(),
                });
            }
        }
        errors
    }

    fn extract_sync_progress(&self, _lines: &[&str]) -> Option<SyncProgress> {
        None // neo-rs doesn't emit explicit sync progress yet
    }
}

#[derive(Debug, Clone)]
pub struct NeoXGethLogParser;

impl LogParserAdapter for NeoXGethLogParser {
    fn parse_line(&self, line: &str) -> Option<LogEntry> {
        let trimmed = line.trim();

        // Try JSON first
        if trimmed.starts_with('{') {
            if let Ok(entry) = serde_json::from_str::<serde_json::Value>(trimmed) {
                let ts = entry.get("time").and_then(|t| t.as_i64()).unwrap_or(0) as u64;
                let level = entry
                    .get("level")
                    .and_then(|l| l.as_str())
                    .unwrap_or("info");
                let msg = entry.get("msg").and_then(|m| m.as_str()).unwrap_or("");
                let module = entry
                    .get("logger")
                    .and_then(|l| l.as_str())
                    .map(|s| s.to_string());

                return Some(LogEntry {
                    timestamp: ts.max(1),
                    level: level.to_uppercase(),
                    message: msg.to_string(),
                    source: module,
                    metadata: entry,
                });
            }
        }

        // Fallback to text format
        let level = extract_kv_value(trimmed, "level")
            .unwrap_or("INFO")
            .to_uppercase();
        let msg = extract_kv_value(trimmed, "msg").unwrap_or("").to_string();
        let ts_raw = extract_kv_value(trimmed, "ts").or_else(|| extract_kv_value(trimmed, "time"));

        let timestamp = ts_raw.and_then(|t| t.parse::<i64>().ok()).unwrap_or(0) as u64;

        Some(LogEntry {
            timestamp: timestamp.max(1),
            level,
            message: msg,
            source: None,
            metadata: serde_json::json!({"parser": "neox-geth", "fallback": true}),
        })
    }

    fn detect_fatal_errors(&self, log_content: &str) -> Vec<FatalError> {
        let mut errors = Vec::new();
        for (line_num, line) in log_content.lines().enumerate() {
            if contains_any_case(
                line,
                &[
                    "crit [",
                    "crit:",
                    "fatal:",
                    "fatal ",
                    "levelfatal",
                    "level=fatal",
                    "level=crit",
                    "lvl=crit",
                    "lvl=fatal",
                    "\"level\":\"fatal\"",
                    "\"level\":\"crit\"",
                ],
            ) {
                errors.push(FatalError {
                    line_number: (line_num + 1) as u32,
                    pattern: "FATAL".to_string(),
                    suggestion: "Check Geth-specific error logs for database or consensus issues"
                        .to_string(),
                });
            }
        }
        errors
    }

    fn extract_sync_progress(&self, lines: &[&str]) -> Option<SyncProgress> {
        for line in lines.iter().rev().take(10) {
            if line.to_lowercase().contains("chain") && line.to_lowercase().contains("block") {
                if let Some(progress) = extract_geth_block_height(line) {
                    return Some(progress);
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct NeoXRethLogParser;

impl LogParserAdapter for NeoXRethLogParser {
    fn parse_line(&self, line: &str) -> Option<LogEntry> {
        if line.starts_with('{') {
            if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
                return parse_reth_json_entry(&entry);
            }
        }

        let timestamp = extract_iso_timestamp(line)?;
        let level_upper = find_log_level(line);
        let message = line.split(':').nth(2).unwrap_or(line).trim().to_string();

        Some(LogEntry {
            timestamp,
            level: level_upper,
            message,
            source: None,
            metadata: serde_json::json!({"parser": "neox-reth", "format": "text"}),
        })
    }

    fn detect_fatal_errors(&self, log_content: &str) -> Vec<FatalError> {
        let mut errors = Vec::new();
        for (line_num, line) in log_content.lines().enumerate() {
            if line.to_lowercase().contains("panicked at")
                || ((line.contains("reth:") || line.contains("reth"))
                    && contains_any_case(line, &["fatal", "crash"]))
            {
                errors.push(FatalError {
                    line_number: (line_num + 1) as u32,
                    pattern: "Reth Fatal".to_string(),
                    suggestion: "Inspect Reth-specific panics and verify MDBX database integrity"
                        .to_string(),
                });
            }
        }
        errors
    }

    fn extract_sync_progress(&self, lines: &[&str]) -> Option<SyncProgress> {
        for line in lines.iter().rev().take(10) {
            if line.contains("Block #") && line.contains("state root") {
                if let (Some(current), Some(target)) = (
                    extract_reth_height(line),
                    extract_bracket_number(line, "targetheight="),
                ) {
                    if let Some(progress) =
                        measured_sync_progress(current, target, extract_peer_count(line))
                    {
                        return Some(progress);
                    }
                }
            }
        }
        None
    }
}

/// Backward-compatibility stub used when no concrete log parser is registered
/// for a node type. Part of the public adapter surface from the phased rollout.
#[allow(dead_code)]
#[derive(Debug, Default, Clone)]
pub struct NoOpLogParserAdapter;

impl LogParserAdapter for NoOpLogParserAdapter {
    fn parse_line(&self, _line: &str) -> Option<LogEntry> {
        None
    }

    fn detect_fatal_errors(&self, _log_content: &str) -> Vec<FatalError> {
        vec![]
    }

    fn extract_sync_progress(&self, _lines: &[&str]) -> Option<SyncProgress> {
        None
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/supervisor/log_parsers.rs"]
mod tests;
