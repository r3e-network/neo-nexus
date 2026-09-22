//! Unit tests for supervisor log parsers across all node engines.

use super::*;

#[test]
fn neo_rs_log_parser_detects_rust_panics_and_fatal_errors() {
    let parser = NeoRsLogParser;

    let normal_logs = "2026-09-12T00:00:00.000Z INFO neo_rs::node: Starting node\n2026-09-12T00:00:01.000Z DEBUG neo_rs::p2p: Connected to peer";
    let normal_errors = parser.detect_fatal_errors(normal_logs);
    assert!(
        normal_errors.is_empty(),
        "normal logs should not yield fatal errors"
    );

    let panic_logs = "thread 'main' panicked at 'assertion failed: state_root == expected', src/state.rs:123:5\nstack backtrace:";
    let panic_errors = parser.detect_fatal_errors(panic_logs);
    assert_eq!(
        panic_errors.len(),
        1,
        "must detect Rust panic in neo-rs logs"
    );
    assert_eq!(panic_errors[0].pattern, "PANIC/FATAL");
    assert!(panic_errors[0].suggestion.contains("panic"));

    let fatal_logs = "2026-09-12T00:00:00.000Z fatal: Database corruption detected";
    let fatal_errors = parser.detect_fatal_errors(fatal_logs);
    assert_eq!(fatal_errors.len(), 1, "must detect FATAL log entry");
    assert_eq!(fatal_errors[0].pattern, "PANIC/FATAL");
}

#[test]
fn neo_x_geth_log_parser_detects_crit_and_fatal_errors() {
    let parser = NeoXGethLogParser;

    let normal_logs = "INFO [09-12|00:00:00.000] Imported new chain segment blocks=1 txs=10";
    let normal_errors = parser.detect_fatal_errors(normal_logs);
    assert!(
        normal_errors.is_empty(),
        "normal geth logs should not yield fatal errors"
    );

    let crit_logs = "CRIT [09-12|00:00:00.000] Failed to store block in database err=\"disk full\"";
    let crit_errors = parser.detect_fatal_errors(crit_logs);
    assert_eq!(crit_errors.len(), 1, "must detect CRIT log entry");
    assert_eq!(crit_errors[0].pattern, "FATAL");

    let lvl_crit_logs = "t=2026-09-12T00:00:00+0000 lvl=crit msg=\"Fatal database crash\"";
    let lvl_crit_errors = parser.detect_fatal_errors(lvl_crit_logs);
    assert_eq!(lvl_crit_errors.len(), 1, "must detect lvl=crit log entry");
    assert_eq!(lvl_crit_errors[0].pattern, "FATAL");
}

#[test]
fn neo_x_reth_log_parser_detects_panics_and_crit_errors() {
    let parser = NeoXRethLogParser;

    let normal_logs = "2026-09-12T00:00:00.000000Z  INFO reth::node: Block execution finished";
    let normal_errors = parser.detect_fatal_errors(normal_logs);
    assert!(
        normal_errors.is_empty(),
        "normal reth logs should not yield fatal errors"
    );

    let panic_logs =
        "thread 'reth-worker-0' panicked at 'storage invariant violated', crates/storage/db.rs:456";
    let panic_errors = parser.detect_fatal_errors(panic_logs);
    assert_eq!(panic_errors.len(), 1, "must detect panic in reth worker");
    assert_eq!(panic_errors[0].pattern, "Reth Fatal");

    let crit_logs = "reth: fatal database corruption encountered";
    let crit_errors = parser.detect_fatal_errors(crit_logs);
    assert_eq!(crit_errors.len(), 1, "must detect fatal in reth");
    assert_eq!(crit_errors[0].pattern, "Reth Fatal");
}

/// A neo-cli start that dies binding its RPC port: the third line is the one an
/// operator has to be sent to, and the log collector quotes its line number.
const NEO_CLI_BIND_FAILURE: &str = "\
2024-01-15T11:00:00Z [INFO] Neo CLI v2.10.0 starting...
2024-01-15T11:00:01Z [INFO] Network: Private Net
2024-01-15T11:00:02Z [ERROR] FATAL: Unable to bind to RPC port 30333 - Address already in use
2024-01-15T11:00:02Z [INFO] Attempting graceful shutdown...
";

/// A healthy neo-cli sync, including a WARN line that is not fatal.
const NEO_CLI_HEALTHY_SYNC: &str = "\
2024-01-15T10:30:00Z [INFO] Neo CLI v2.10.0 starting...
2024-01-15T10:30:04Z [DEBUG] Loaded 12 plugins from Plugins directory
2024-01-15T10:30:10Z [INFO] Syncing block 2847001/2847392 (0.3%) - peers: 5
2024-01-15T10:30:20Z [WARN] Slow consensus round took 2500ms
2024-01-15T10:30:35Z [INFO] Synchronized to the blockchain
";

/// A neo-go run with a failed transaction (an ERROR, not a fatal) and then a
/// fatal RPC bind.
const NEO_GO_RUN: &str = "\
2024-01-15T10:30:00.000Z\tINFO\tstarting NeoGo node\t{\"network\": \"TestNet\"}
2024-01-15T10:30:15.000Z\tERROR\ttransaction validation failed: insufficient fee
2024-01-15T10:30:33.000Z\tFATAL\tfailed to start RPC server\t{\"error\": \"listen tcp :30333: bind: address already in use\"}
";

/// A neo-rs panic followed by its backtrace: one fatal event, not one per frame.
const NEO_RS_PANIC_WITH_BACKTRACE: &str = "\
2024-01-15T11:00:00.123Z INFO  neo_node::main[1234] Neo Node v0.9.5 starting
2024-01-15T11:00:01.234Z DEBUG neo_node::config[1234] Configuration loaded
thread '<unnamed>' panicked at src/blockchain.rs:245: integer overflow during block number calculation
stack backtrace:
   0: rust_begin_unwind
   1: core::panicking::panic_fmt
   2: neo_node::blockchain::BlockChain::apply_block
   3: neo_node::sync::SyncManager::process_block
2024-01-15T11:00:02.345Z ERROR neo_node::fatal[1234] Node crashed unexpectedly
";

#[test]
fn neo_cli_log_parser_reports_the_line_a_fatal_bind_failure_is_on() {
    let errors = NeoCliLogParser.detect_fatal_errors(NEO_CLI_BIND_FAILURE);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].line_number, 3);
    assert_eq!(errors[0].pattern, "FATAL/PANIC");

    assert!(NeoCliLogParser
        .detect_fatal_errors(NEO_CLI_HEALTHY_SYNC)
        .is_empty());
}

#[test]
fn neo_go_log_parser_tells_a_fatal_from_an_error() {
    let errors = NeoGoLogParser.detect_fatal_errors(NEO_GO_RUN);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].line_number, 3);
    assert_eq!(errors[0].pattern, "FATAL/PANIC");
}

#[test]
fn neo_rs_log_parser_reports_a_panic_once_at_its_own_line() {
    let errors = NeoRsLogParser.detect_fatal_errors(NEO_RS_PANIC_WITH_BACKTRACE);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].line_number, 3);
    assert_eq!(errors[0].pattern, "PANIC/FATAL");
}
