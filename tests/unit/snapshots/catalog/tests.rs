use super::FastSyncSnapshotCatalog;
use crate::{
    snapshots::FastSyncSnapshotManager,
    types::{Network, NodeType},
};

#[test]
fn catalog_entry_trims_values_and_falls_back_to_url_file_name() -> anyhow::Result<()> {
    let catalog_text = serde_json::json!({
        "schema_version": 1,
        "generated_at_unix": 1_800_000_000u64,
        "snapshots": [
            {
                "id": " neo-rs-testnet ",
                "label": " Neo RS Testnet ",
                "network": " testnet ",
                "node_type": " neo-rs ",
                "url": " https://snapshots.example.com/neo-rs-testnet.acc ",
                "download_file_name": " ",
                "expected_sha256": format!(" {} ", "a".repeat(64))
            }
        ]
    })
    .to_string();

    let catalog = FastSyncSnapshotCatalog::from_json(&catalog_text)?;
    let entry = &catalog.snapshots[0];

    assert_eq!(catalog.generated_at_unix, Some(1_800_000_000));
    assert_eq!(entry.id, "neo-rs-testnet");
    assert_eq!(entry.label, "Neo RS Testnet");
    assert_eq!(entry.network, Network::Testnet);
    assert_eq!(entry.node_type, NodeType::NeoRs);
    assert_eq!(entry.file_name, "neo-rs-testnet.acc");
    assert_eq!(entry.expected_sha256, "a".repeat(64));
    assert_eq!(
        entry.max_bytes,
        FastSyncSnapshotManager::DEFAULT_DOWNLOAD_MAX_BYTES
    );

    Ok(())
}
