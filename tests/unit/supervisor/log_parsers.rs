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
