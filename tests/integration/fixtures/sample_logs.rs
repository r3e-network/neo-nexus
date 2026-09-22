//! Sample log files for parser testing
//!
//! Provides realistic log samples for each node type's expected format and
//! patterns that parsers need to handle (including edge cases).

/// NeoCli-style log entries (C# formatted)
pub fn neo_cli_logs() -> &'static str {
    r#"2024-01-15T10:30:00Z [INFO] Neo CLI v2.10.0 starting...
2024-01-15T10:30:01Z [DEBUG] Loading configuration from config.json
2024-01-15T10:30:01Z [INFO] Network: Private Net
2024-01-15T10:30:01Z [INFO] Storage Engine: RocksDB
2024-01-15T10:30:02Z [INFO] Blockchain initialized, current height: 2847000
2024-01-15T10:30:03Z [INFO] P2P peer discovery started
2024-01-15T10:30:04Z [DEBUG] Loaded 12 plugins from Plugins directory
2024-01-15T10:30:05Z [INFO] RPC server started on port 30333
2024-01-15T10:30:10Z [INFO] Syncing block 2847001/2847392 (0.3%) - peers: 5
2024-01-15T10:30:15Z [INFO] Syncing block 2847100/2847392 (3.5%) - peers: 8
2024-01-15T10:30:20Z [WARN] Slow consensus round took 2500ms
2024-01-15T10:30:25Z [INFO] Syncing block 2847200/2847392 (69.2%) - peers: 12
2024-01-15T10:30:30Z [INFO] Syncing block 2847300/2847392 (95.8%) - peers: 15
2024-01-15T10:30:35Z [INFO] Synchronized to the blockchain
2024-01-15T10:30:36Z [INFO] Node ready, accepting connections
"#
}

/// NeoGo-style log entries (Go formatted)
pub fn neo_go_logs() -> &'static str {
    r#"[01/15/24 10:30:00] INFO  node.go:123 Starting NeoGo node
[01/15/24 10:30:00] DEBUG config.go:45 Reading config from config.yml
[01/15/24 10:30:01] INFO  rpc.go:234 Network: MainNet
[01/15/24 10:30:01] INFO  storage.go:89 Using LevelDB as storage backend
[01/15/24 10:30:02] INFO  blockchain.go:67 Block 0 verified successfully
[01/15/24 10:30:03] WARN  p2p.go:178 Peer connection rate limited
[01/15/24 10:30:05] INFO  sync.go:245 Syncing block 2847001/2847392 (~0.3%) - peers: 4
[01/15/24 10:30:10] INFO  sync.go:245 Syncing block 2847100/2847392 (~3.5%) - peers: 7
[01/15/24 10:30:15] ERROR transaction.go:312 Transaction validation failed: insufficient fee
[01/15/24 10:30:20] INFO  sync.go:245 Syncing block 2847200/2847392 (~69.1%) - peers: 10
[01/15/24 10:30:25] INFO  sync.go:245 Syncing block 2847300/2847392 (~95.7%) - peers: 14
[01/15/24 10:30:30] INFO  sync.go:245 Block 2847392 synced - node is fully synchronized
[01/15/24 10:30:31] INFO  rpc.go:456 RPC server listening on :30333
[01/15/24 10:30:32] DEBUG metrics.go:78 Prometheus metrics exported at :2112
"#
}

/// NeoRs-style log entries (Rust formatted)
pub fn neo_rs_logs() -> &'static str {
    r#"2024-01-15T10:30:00.123Z INFO  neo_node::main[1234] Neo Node v0.9.5 starting
2024-01-15T10:30:00.145Z DEBUG neo_node::config[1234] Loading configuration from config/config.json
2024-01-15T10:30:00.234Z INFO  neo_node::chain[1234] Chain network: TestNet
2024-01-15T10:30:00.345Z INFO  neo_node::storage[1234] Storage engine initialized: RocksDB
2024-01-15T10:30:00.567Z INFO  neo_node::blockchain[1234] Genesis block loaded: hash=0x0000...
2024-01-15T10:30:01.123Z INFO  neo_node::p2p[1234] P2P handshake completed with bootstrap nodes
2024-01-15T10:30:02.234Z WARN  neo_node::consensus[1234] Consensus round timeout: no quorum reached
2024-01-15T10:30:05.567Z INFO  neo_node::sync[1234] Sync progress: block 2847001/2847392 (0.3%) - connected_peers=6
2024-01-15T10:30:10.123Z INFO  neo_node::sync[1234] Sync progress: block 2847100/2847392 (3.5%) - connected_peers=9
2024-01-15T10:30:15.234Z DEBUG neo_node::txpool[1234] Transaction pool size: 47 transactions
2024-01-15T10:30:20.345Z INFO  neo_node::sync[1234] Sync progress: block 2847200/2847392 (69.2%) - connected_peers=13
2024-01-15T10:30:25.567Z INFO  neo_node::sync[1234] Sync progress: block 2847300/2847392 (95.8%) - connected_peers=17
2024-01-15T10:30:30.123Z INFO  neo_node::sync[1234] Synchronization complete - head block reached
2024-01-15T10:30:31.234Z INFO  neo_node::rpc[1234] JSON-RPC server started on 127.0.0.1:30333
2024-01-15T10:30:32.345Z INFO  neo_node::metrics[1234] Metrics endpoint available at http://localhost:9090/metrics
"#
}

/// Logs containing fatal errors (for error detection testing)
pub fn neo_cli_with_fatal_errors() -> &'static str {
    r#"2024-01-15T11:00:00Z [INFO] Neo CLI v2.10.0 starting...
2024-01-15T11:00:01Z [INFO] Network: Private Net
2024-01-15T11:00:02Z [ERROR] FATAL: Unable to bind to RPC port 30333 - Address already in use
2024-01-15T11:00:02Z [INFO] Attempting graceful shutdown...
"#
}

/// Logs with panic scenarios (Rust panics)
pub fn neo_rs_with_panic() -> &'static str {
    r#"2024-01-15T11:00:00.123Z INFO  neo_node::main[1234] Neo Node v0.9.5 starting
2024-01-15T11:00:01.234Z DEBUG neo_node::config[1234] Configuration loaded
thread '<unnamed>' panicked at src/blockchain.rs:245: integer overflow during block number calculation
stack backtrace:
   0: rust_begin_unwind
   1: core::panicking::panic_fmt
   2: neo_node::blockchain::BlockChain::apply_block
   3: neo_node::sync::SyncManager::process_block
2024-01-15T11:00:02.345Z ERROR neo_node::fatal[1234] Node crashed unexpectedly
"#
}

/// Empty/minimal logs (edge case for parser)
pub fn minimal_logs() -> &'static str {
    "2024-01-15T10:30:00Z [INFO] Node started\n"
}

/// Very long log file (performance test)
pub fn long_log_file(num_lines: usize) -> String {
    let mut result = String::new();

    for i in 0..num_lines {
        let percentage = if num_lines > 0 {
            ((i as f32 / num_lines as f32) * 100.0).round()
        } else {
            0.0
        };

        result.push_str(&format!(
            "2024-01-15T10:30:{:02.0}Z [INFO] Syncing block 2847{}00/2847392 ({:.1}%) - peers: {}\n",
            (i % 60) as f32 / 10.0,
            i % 1000,
            percentage,
            5 + (i % 20)
        ));
    }

    result.push_str("2024-01-15T10:31:00Z [INFO] Synchronized to the blockchain\n");
    result
}

/// Log with mixed severity levels
pub fn mixed_severity_logs() -> &'static str {
    r#"2024-01-15T10:30:00Z [INFO] Node initialization started
2024-01-15T10:30:01Z [TRACE] Loading component: Database
2024-01-15T10:30:02Z [DEBUG] Component loaded: Database
2024-01-15T10:30:03Z [INFO] Connecting to database...
2024-01-15T10:30:04Z [WARN] Database connection slow (>1000ms)
2024-01-15T10:30:05Z [ERROR] Failed to connect: Connection refused
2024-01-15T10:30:06Z [INFO] Retrying connection (attempt 2/3)...
2024-01-15T10:30:07Z [INFO] Database connected successfully
2024-01-15T10:30:08Z [FATAL] Critical configuration error: invalid chain parameters
2024-01-15T10:30:09Z [INFO] Shutting down due to critical error
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neo_cli_logs_structure() {
        let logs = neo_cli_logs();
        let lines: Vec<&str> = logs.lines().collect();

        assert!(!lines.is_empty());
        assert!(lines.iter().any(|l| l.contains("starting")));
        assert!(lines.iter().any(|l| l.contains("Syncing")));
        assert!(lines.iter().any(|l| l.contains("Synchronized")));
    }

    #[test]
    fn test_neo_go_logs_format() {
        let logs = neo_go_logs();

        // Go log format is `[MM/DD/YY HH:MM:SS] LEVEL component:line message`,
        // so the level sits outside the bracketed timestamp.
        assert!(logs.contains("INFO  node.go:123"));
        assert!(logs.contains("Syncing block"));
        assert!(logs.contains("node is fully synchronized"));
    }

    #[test]
    fn test_neo_rs_logs_format() {
        let logs = neo_rs_logs();

        // Rust log format with module paths
        assert!(logs.contains("neo_node::"));
        assert!(logs.contains("Sync progress:"));
        assert!(logs.contains("Synchronization complete"));
    }

    #[test]
    fn test_fatal_error_detection() {
        let logs = neo_cli_with_fatal_errors();

        assert!(logs.contains("FATAL"));
        assert!(logs.contains("Address already in use"));
        assert!(logs.lines().count() < 10); // Short error log
    }

    #[test]
    fn test_panic_detection() {
        let logs = neo_rs_with_panic();

        assert!(logs.contains("panicked at"));
        assert!(logs.contains("integer overflow"));
        assert!(logs.contains("fatal"));
    }

    #[test]
    fn test_minimal_logs_edge_case() {
        let logs = minimal_logs();

        assert_eq!(logs.lines().count(), 1);
        assert!(logs.contains("Node started"));
    }

    #[test]
    fn test_long_log_performance() {
        let long_logs = long_log_file(1000);
        let line_count = long_logs.lines().count();

        assert!(line_count >= 1000);
        assert!(long_logs.contains("Synchronized to the blockchain"));
    }

    #[test]
    fn test_mixed_severity_parsing() {
        let logs = mixed_severity_logs();
        let lines: Vec<&str> = logs.lines().collect();

        assert!(lines.iter().any(|l| l.contains("[TRACE")));
        assert!(lines.iter().any(|l| l.contains("[DEBUG")));
        assert!(lines.iter().any(|l| l.contains("[INFO")));
        assert!(lines.iter().any(|l| l.contains("[WARN")));
        assert!(lines.iter().any(|l| l.contains("[ERROR")));
        assert!(lines.iter().any(|l| l.contains("[FATAL")));
    }

    #[test]
    fn test_sync_progress_patterns() {
        let cli_logs = neo_cli_logs();
        let rs_logs = neo_rs_logs();

        // Both formats should have sync patterns
        assert!(cli_logs.contains("Syncing block"));
        assert!(rs_logs.contains("Sync progress:"));

        // Both should show progression from 0% to completion
        let lines: Vec<&str> = cli_logs.lines().collect();
        let last_info_line = lines
            .iter()
            .rev()
            .find(|l| l.contains("[INFO]") && l.contains("Syncing"))
            .unwrap();

        assert!(last_info_line.contains("95.8%"));
    }
}
