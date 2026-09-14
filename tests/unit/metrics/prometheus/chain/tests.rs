use super::*;

fn answering() -> ChainMetricRow {
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
        rpc_latency_ms: Some(250),
        head_lag_blocks: Some(0),
        seconds_since_height_changed: Some(12),
        seconds_since_head_block: Some(3),
        observed_magic: Some(1_230_000),
        sampled_at_unix: Some(1_770_000_000),
    }
}

fn render(rows: &[ChainMetricRow]) -> String {
    let mut output = String::new();
    push_chain_metrics(&mut output, rows);
    output
}

/// The three names `docs/AGENT_API.md` has always documented, and which had
/// zero hits in `src/` — so an operator who wired an alert to them got a rule
/// that never fired.
#[test]
fn the_documented_metric_names_exist() {
    let text = render(&[answering()]);
    for name in [
        "neonexus_node_running",
        "neonexus_node_block_height",
        "neonexus_node_rpc_latency_seconds",
    ] {
        assert!(
            text.contains(&format!("# TYPE {name} gauge")),
            "{name}\n{text}"
        );
        assert!(text.contains(&format!("{name}{{")), "{name}\n{text}");
    }
}

/// A `_seconds` metric carrying milliseconds is a trap for whoever writes the
/// first alert against it.
#[test]
fn latency_is_published_in_the_unit_its_name_promises() {
    let text = render(&[answering()]);
    assert!(
        text.contains("neonexus_node_rpc_latency_seconds{node_id=\"node-1\""),
        "{text}"
    );
    assert!(text.contains(" 0.25\n"), "250ms must be 0.25s: {text}");
}

/// **A value that was not read is not emitted.**
///
/// Prometheus has no null; a missing series is how it says "no data". Emitting
/// zero for an unread peer count would put a node whose client does not
/// implement `getconnectioncount` permanently inside whatever alert watches for
/// isolation.
#[test]
fn an_unread_value_produces_no_series_rather_than_a_zero() {
    let unread = ChainMetricRow {
        peers_connected: None,
        block_height: None,
        rpc_latency_ms: None,
        head_lag_blocks: None,
        seconds_since_height_changed: None,
        ..answering()
    };
    let text = render(&[unread]);

    for name in [
        "neonexus_node_peers_connected",
        "neonexus_node_block_height",
        "neonexus_node_rpc_latency_seconds",
        "neonexus_node_head_lag_blocks",
        "neonexus_node_seconds_since_height_changed",
    ] {
        assert!(
            text.contains(&format!("# TYPE {name} gauge")),
            "the family is still declared: {name}"
        );
        assert!(
            !text.contains(&format!("{name}{{")),
            "{name} emitted a sample for a value nobody read:\n{text}"
        );
    }
    // What *is* always known still appears.
    assert!(text.contains("neonexus_node_running{"), "{text}");
}

/// Zero is a reading and must survive as one. A node with no peers is
/// `Isolated`; that series has to exist for an alert to fire on it.
#[test]
fn a_measured_zero_is_published() {
    let isolated = ChainMetricRow {
        peers_connected: Some(0),
        ..answering()
    };
    let text = render(&[isolated]);
    assert!(text.contains("neonexus_node_peers_connected{"), "{text}");
    assert!(
        text.trim_end().ends_with(" 0") || text.contains("\"} 0\n"),
        "{text}"
    );
}

/// A node nobody has judged has no health series. `Unknown` is a verdict
/// reached by looking; no verdict at all is not, and a scrape must not fuse
/// them.
#[test]
fn a_node_with_no_verdict_publishes_no_health_series() {
    let unjudged = ChainMetricRow {
        health_state: None,
        ..answering()
    };
    let text = render(&[unjudged]);
    assert!(!text.contains("neonexus_node_health{"), "{text}");

    let judged = render(&[answering()]);
    assert!(judged.contains(r#"state="healthy""#), "{judged}");
    assert!(judged.contains("neonexus_node_health{"), "{judged}");
}

/// Series are keyed by the chain a node **joined**, not the network it was
/// configured with. Two nodes labelled `network="private"` can be on different
/// chains — one having fallen back to compiled-in MainNet defaults — and
/// summing their heights would be meaningless.
#[test]
fn the_chain_label_carries_the_magic_the_node_actually_joined() {
    let text = render(&[answering()]);
    assert!(text.contains(r#"chain="1230000""#), "{text}");

    // A node that has not reported its magic is not labelled with a guess.
    let unknown_chain = ChainMetricRow {
        observed_magic: None,
        ..answering()
    };
    assert!(!render(&[unknown_chain]).contains("chain="));
}

/// An empty fleet emits nothing at all, rather than a page of headers with no
/// samples under them.
#[test]
fn no_nodes_produces_no_output() {
    assert!(render(&[]).is_empty());
}
