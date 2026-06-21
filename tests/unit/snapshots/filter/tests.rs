use std::path::PathBuf;

use crate::types::{Network, NodeType};

use super::*;

#[test]
fn snapshot_filter_matches_runtime_network_state_and_query() {
    let snapshots = [
        snapshot(
            "neo-rs-testnet",
            Network::Testnet,
            NodeType::NeoRs,
            true,
            true,
        ),
        snapshot(
            "neo-cli-mainnet",
            Network::Mainnet,
            NodeType::NeoCli,
            false,
            false,
        ),
    ];
    let filter = SnapshotFilter::new(
        Some(Network::Testnet),
        Some(NodeType::NeoRs),
        Some(true),
        Some(true),
        "neo-rs",
    );

    let filtered = filter_snapshots(&snapshots, &filter);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, "neo-rs-testnet");
}

#[test]
fn snapshot_catalog_filter_matches_runtime_network_and_query() {
    let entries = [
        entry("neo-rs-testnet", Network::Testnet, NodeType::NeoRs),
        entry("neo-go-mainnet", Network::Mainnet, NodeType::NeoGo),
    ];
    let filter = SnapshotCatalogEntryFilter::new(
        Some(Network::Testnet),
        Some(NodeType::NeoRs),
        "neo-rs-testnet.acc",
    );

    let filtered = filter_snapshot_catalog_entries(&entries, &filter);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, "neo-rs-testnet");
}

fn snapshot(
    id: &str,
    network: Network,
    node_type: NodeType,
    verified: bool,
    cached: bool,
) -> FastSyncSnapshot {
    FastSyncSnapshot {
        id: id.to_string(),
        label: format!("{id} snapshot"),
        network,
        node_type,
        source_path: PathBuf::from(format!("/snapshots/{id}.acc")),
        source_url: Some(format!("https://snapshots.example.com/{id}.acc")),
        download_file_name: Some(format!("{id}.acc")),
        download_max_bytes: 2048,
        expected_sha256: "a".repeat(64),
        cached_path: cached.then(|| PathBuf::from(format!("/cache/{id}.acc"))),
        verified_sha256: verified.then(|| "a".repeat(64)),
        verified_at_unix: verified.then_some(1_800_000_000),
        bytes: Some(1024),
    }
}

fn entry(id: &str, network: Network, node_type: NodeType) -> FastSyncSnapshotCatalogEntry {
    FastSyncSnapshotCatalogEntry {
        id: id.to_string(),
        label: format!("{id} catalog"),
        network,
        node_type,
        url: format!("https://snapshots.example.com/{id}.acc"),
        file_name: format!("{id}.acc"),
        expected_sha256: "b".repeat(64),
        max_bytes: 4096,
    }
}
