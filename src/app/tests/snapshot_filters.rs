use super::*;

#[test]
fn snapshot_filters_limit_registry_and_catalog_selection() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);
    let snapshots = vec![
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

    app.snapshot_query = "neo-rs".to_string();
    app.snapshot_network_filter = Some(Network::Testnet);
    app.snapshot_type_filter = Some(NodeType::NeoRs);
    app.snapshot_verified_filter = Some(true);
    app.snapshot_cached_filter = Some(true);
    let visible_snapshots = app.filtered_snapshots(&snapshots);
    assert_eq!(visible_snapshots.len(), 1);
    assert_eq!(visible_snapshots[0].id, "neo-rs-testnet");

    app.selected_snapshot = Some("neo-cli-mainnet".to_string());
    app.snapshot_page = 5;
    app.ensure_valid_snapshot_selection(&snapshots);
    assert_eq!(app.selected_snapshot.as_deref(), Some("neo-rs-testnet"));
    assert_eq!(app.snapshot_page, 0);

    app.snapshot_catalog = Some(FastSyncSnapshotCatalog {
        schema_version: 1,
        generated_at_unix: Some(1_800_000_000),
        snapshots: vec![
            catalog_entry("neo-rs-catalog", Network::Testnet, NodeType::NeoRs),
            catalog_entry("neo-go-catalog", Network::Mainnet, NodeType::NeoGo),
        ],
    });
    app.snapshot_catalog_query = "neo-rs".to_string();
    app.snapshot_catalog_network_filter = Some(Network::Testnet);
    app.snapshot_catalog_type_filter = Some(NodeType::NeoRs);
    app.selected_snapshot_catalog_entry = Some("neo-go-catalog".to_string());
    app.snapshot_catalog_page = 4;
    app.ensure_valid_snapshot_catalog_selection();

    assert_eq!(
        app.selected_snapshot_catalog_entry.as_deref(),
        Some("neo-rs-catalog")
    );
    assert_eq!(app.snapshot_catalog_page, 0);

    Ok(())
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
        download_max_bytes: 4096,
        expected_sha256: "a".repeat(64),
        cached_path: cached.then(|| PathBuf::from(format!("/cache/{id}.acc"))),
        verified_sha256: verified.then(|| "a".repeat(64)),
        verified_at_unix: verified.then_some(1_800_000_000),
        bytes: Some(2048),
    }
}

fn catalog_entry(id: &str, network: Network, node_type: NodeType) -> FastSyncSnapshotCatalogEntry {
    FastSyncSnapshotCatalogEntry {
        id: id.to_string(),
        label: format!("{id} catalog"),
        network,
        node_type,
        url: format!("https://snapshots.example.com/{id}.acc"),
        file_name: format!("{id}.acc"),
        expected_sha256: "b".repeat(64),
        max_bytes: 8192,
    }
}
