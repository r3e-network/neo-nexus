//! Every metric the documentation names must exist in the exposition.
//!
//! `docs/AGENT_API.md` documented `neonexus_node_running`,
//! `neonexus_node_block_height` and `neonexus_node_rpc_latency_seconds` for as
//! long as the file existed, and all three had zero hits in `src/`. Nothing
//! failed. An operator who wired an alert to a documented name got a rule that
//! never fired — the worst kind of monitoring, because it looks like coverage.
//!
//! This reads the documentation and checks the exposition, so the two cannot
//! part company again without the build saying so.

use std::{collections::BTreeSet, fs, path::Path};

use crate::metrics::{exposition, ChainMetricRow, MetricsCollector};

/// Every `neonexus_*` name the Prometheus section of the docs mentions, from
/// its sample output and from the prose beneath it alike.
///
/// Scoped to that one section on purpose: `neonexus_session` is a cookie name
/// elsewhere in the same file, and a check that cannot tell a metric from a
/// cookie would either fail forever or be turned off.
fn documented_names() -> BTreeSet<String> {
    const SECTION: &str = "### GET /api/metrics-prometheus";
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/AGENT_API.md");
    let text = fs::read_to_string(path).expect("the agent API documentation is part of the repo");
    let start = text
        .find(SECTION)
        .expect("the documentation still describes the Prometheus endpoint");
    let body = &text[start + SECTION.len()..];
    let body = body.find("\n---").map_or(body, |end| &body[..end]);

    let mut names = BTreeSet::new();
    for token in body.split(|c: char| !(c.is_alphanumeric() || c == '_')) {
        if token.starts_with("neonexus_") {
            names.insert(token.to_string());
        }
    }
    assert!(
        !names.is_empty(),
        "no metric names were found in the documentation; this test is looking in the wrong place"
    );
    names
}

/// One node with every field populated, so no family is skipped for want of a
/// value and the check is about existence rather than about this fixture.
fn fully_populated_row() -> ChainMetricRow {
    ChainMetricRow {
        node_id: "node-1".to_string(),
        node_name: "rpc-1".to_string(),
        client: "neo-go".to_string(),
        network: "private".to_string(),
        health_state: Some("healthy".to_string()),
        process_running: true,
        block_height: Some(8_421),
        header_height: Some(8_421),
        peers_connected: Some(6),
        rpc_latency_ms: Some(11),
        head_lag_blocks: Some(0),
        seconds_since_height_changed: Some(12),
        seconds_since_head_block: Some(3),
        observed_magic: Some(1_230_000),
        sampled_at_unix: Some(1_770_000_000),
    }
}

#[test]
fn every_documented_metric_is_actually_exposed() {
    let snapshot =
        MetricsCollector::new(std::time::Duration::ZERO).refresh(&[], std::time::Instant::now());
    let text = exposition(&snapshot, &[fully_populated_row()]);

    let missing: Vec<String> = documented_names()
        .into_iter()
        .filter(|name| !text.contains(&format!("# TYPE {name} gauge")))
        .collect();
    assert!(
        missing.is_empty(),
        "docs/AGENT_API.md names metrics the exposition does not publish. \
         Publish them, or stop documenting them:\n  {}",
        missing.join("\n  ")
    );
}
