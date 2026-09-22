//! The adapter registry decides, per node type, which parser reads a node's log
//! (the supervisor's log collector) and where its metrics live (the web API's
//! node metrics endpoint). A type wired to another client's adapter fails
//! quietly in production — its fatal errors reach the operator with the wrong
//! client's advice or not at all, and its metrics link points somewhere else —
//! so the wiring is pinned here for every type.

use super::NodeAdapters;
use crate::types::NodeType;

#[test]
fn every_node_type_has_a_log_parser_and_a_metrics_adapter() {
    let adapters = NodeAdapters::initialized();
    for node_type in NodeType::ALL {
        assert!(
            adapters.has_log_parser(&node_type),
            "{node_type} has no log parser"
        );
        assert!(
            adapters.has_metrics(&node_type),
            "{node_type} has no metrics adapter"
        );
    }
}

/// Each type's registered parser recognises its own client's fatal line and
/// answers with that client's remediation advice.
#[test]
fn each_node_type_is_read_by_its_own_clients_log_parser() {
    let adapters = NodeAdapters::initialized();
    for (node_type, fatal_line, advice) in [
        (
            NodeType::NeoCli,
            "2024-01-15T11:00:02Z [ERROR] FATAL: Unable to bind to RPC port 30333 - Address already in use",
            "database integrity",
        ),
        (
            NodeType::NeoGo,
            "2024-01-15T10:30:33.000Z\tFATAL\tfailed to start RPC server",
            "config.yml",
        ),
        (
            NodeType::NeoRs,
            "thread '<unnamed>' panicked at src/blockchain.rs:245: integer overflow",
            "rust panic backtrace",
        ),
        (
            NodeType::NeoXGeth,
            "CRIT [09-12|12:00:00] Fatal database corruption lvl=crit",
            "Geth",
        ),
        (
            NodeType::NeoXReth,
            "thread 'main' panicked at 'MDBX error'",
            "Reth",
        ),
    ] {
        let parser = adapters
            .get_log_parser(&node_type)
            .unwrap_or_else(|| unreachable!("{node_type} has no log parser"));
        let errors = parser.detect_fatal_errors(fatal_line);
        assert_eq!(
            errors.len(),
            1,
            "{node_type}'s parser missed its own fatal line: {fatal_line}"
        );
        assert!(
            errors[0].suggestion.contains(advice),
            "{node_type} answered with another client's advice: {}",
            errors[0].suggestion
        );
    }
}

/// What the node metrics endpoint reports as `metrics_endpoint`, per type.
#[test]
fn each_node_type_advertises_its_own_metrics_endpoint() {
    let adapters = NodeAdapters::initialized();
    for (node_type, endpoint, exporter) in [
        (
            NodeType::NeoCli,
            Some("http://localhost:9090/metrics"),
            Some("prometheus-net-adapter"),
        ),
        (
            NodeType::NeoGo,
            Some("http://localhost:20332/metrics"),
            Some("prometheus-exporter"),
        ),
        (NodeType::NeoRs, None, None),
        (
            NodeType::NeoXGeth,
            Some("http://localhost:8546/metrics"),
            None,
        ),
        (
            NodeType::NeoXReth,
            Some("http://localhost:9091/metrics"),
            None,
        ),
    ] {
        let adapter = adapters
            .get_metrics_adapter(&node_type)
            .unwrap_or_else(|| unreachable!("{node_type} has no metrics adapter"));
        assert_eq!(
            adapter.metrics_url(20332).as_deref(),
            endpoint,
            "{node_type} metrics endpoint"
        );
        assert_eq!(adapter.exporter_package(), exporter, "{node_type} exporter");
    }

    // neo-go serves metrics on its RPC port, so a node without one has no endpoint.
    let neo_go = adapters
        .get_metrics_adapter(&NodeType::NeoGo)
        .unwrap_or_else(|| unreachable!("neo-go has no metrics adapter"));
    assert_eq!(neo_go.metrics_url(0), None);
}
