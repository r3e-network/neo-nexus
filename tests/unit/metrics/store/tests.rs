use super::*;

use std::path::PathBuf;

use crate::types::{Network, NodeConfig, NodeStatus, NodeType, StorageEngine};

fn nodes() -> Vec<NodeConfig> {
    vec![NodeConfig {
        id: "node-1".to_string(),
        name: "rpc-1".to_string(),
        node_type: NodeType::NeoGo,
        network: Network::Private,
        binary_path: PathBuf::from("/opt/neo/neo-go"),
        args: Vec::new(),
        runtime_version: "0.122".to_string(),
        storage_engine: StorageEngine::LevelDb,
        rpc_port: 30332,
        p2p_port: 30333,
        ws_port: None,
        status: NodeStatus::Stopped,
        pid: None,
    }]
}

/// A page served before the engine's first tick still gets a reading. Returning
/// nothing would leave the surfaces with an absence to render that has nothing
/// to do with the host.
#[test]
fn the_first_reader_gets_a_snapshot_rather_than_nothing() {
    let store = MetricsStore::default();
    let snapshot = store.snapshot(&nodes());
    assert!(snapshot.captured_at_unix > 0);
    assert!(snapshot.system.total_memory_bytes > 0);
    assert_eq!(store.history().len(), 1, "and the reading is kept");
}

/// Readers share the tick's reading rather than each taking their own.
///
/// Two collectors sampling the host at unrelated moments is how the Health page
/// and the fleet overview came to disagree about the same machine — and, worse,
/// how every one of them ended up reporting its constructor's first sample,
/// because a fresh collector refreshed twice inside `sysinfo`'s minimum CPU
/// interval.
#[test]
fn every_reader_sees_the_same_reading() {
    let store = MetricsStore::default();
    let nodes = nodes();
    let first = store.snapshot(&nodes);
    let second = store.snapshot(&nodes);
    assert_eq!(first.captured_at_unix, second.captured_at_unix);
    assert_eq!(
        store.history().len(),
        1,
        "reading twice is not sampling twice"
    );
}

/// The ring is bounded, so a workspace left open for a week does not grow a
/// sample per ten seconds forever.
#[test]
fn the_history_is_bounded_and_keeps_the_newest() {
    let store = MetricsStore::default();
    {
        let mut inner = store.lock();
        for round in 0..(HISTORY_SAMPLES + 25) {
            let mut snapshot = inner.collector.refresh(&[], std::time::Instant::now());
            snapshot.captured_at_unix = 1_770_000_000 + round as u64;
            inner.remember(snapshot);
        }
    }
    let history = store.history();
    assert_eq!(history.len(), HISTORY_SAMPLES);
    assert_eq!(
        history.last().map(|sample| sample.at_unix),
        Some(1_770_000_000 + (HISTORY_SAMPLES + 24) as u64),
        "the newest reading must survive"
    );
    assert!(
        history
            .first()
            .is_some_and(|sample| sample.at_unix > 1_770_000_000),
        "and the oldest must have been dropped"
    );
}

/// Samples come back oldest first, because that is the order a line is drawn
/// in and a chart that plots them backwards is worse than no chart.
#[test]
fn history_reads_oldest_first() {
    let store = MetricsStore::default();
    {
        let mut inner = store.lock();
        for round in 0..4u64 {
            let mut snapshot = inner.collector.refresh(&[], std::time::Instant::now());
            snapshot.captured_at_unix = 1_770_000_000 + round;
            inner.remember(snapshot);
        }
    }
    let stamps: Vec<u64> = store
        .history()
        .into_iter()
        .map(|sample| sample.at_unix)
        .collect();
    assert_eq!(
        stamps,
        vec![1_770_000_000, 1_770_000_001, 1_770_000_002, 1_770_000_003]
    );
}

/// The interval is above `sysinfo`'s minimum, without which consecutive
/// refreshes are discarded and the CPU delta is never recomputed — the defect
/// this store exists to fix.
#[test]
fn the_sample_interval_clears_the_floor_that_froze_every_cpu_figure() {
    assert!(SAMPLE_INTERVAL >= std::time::Duration::from_millis(200));
    assert_eq!(
        SAMPLE_INTERVAL.as_secs() * HISTORY_SAMPLES as u64,
        3_600,
        "the retained window is the hour the chart's axis claims"
    );
}
