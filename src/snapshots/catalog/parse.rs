use std::{collections::BTreeSet, str::FromStr};

use anyhow::{Context, Result};

use super::{
    dto::{SnapshotCatalogDto, SnapshotCatalogEntryDto},
    entry::{FastSyncSnapshotCatalog, FastSyncSnapshotCatalogEntry},
};
use crate::{
    snapshots::{
        manager::FastSyncSnapshotManager,
        validation::{file_name_from_url, validate_snapshot_catalog_entry},
    },
    types::{Network, NodeType},
};

pub(super) fn parse_catalog_json(text: &str) -> Result<FastSyncSnapshotCatalog> {
    let dto: SnapshotCatalogDto =
        serde_json::from_str(text).context("snapshot catalog JSON is invalid")?;
    if dto.schema_version != 1 {
        anyhow::bail!(
            "unsupported snapshot catalog schema version {}",
            dto.schema_version
        );
    }

    let snapshots = parse_catalog_entries(dto.snapshots)?;
    Ok(FastSyncSnapshotCatalog {
        schema_version: dto.schema_version,
        generated_at_unix: dto.generated_at_unix,
        snapshots,
    })
}

fn parse_catalog_entries(
    entries: Vec<SnapshotCatalogEntryDto>,
) -> Result<Vec<FastSyncSnapshotCatalogEntry>> {
    let mut ids = BTreeSet::new();
    let mut snapshots = Vec::with_capacity(entries.len());
    for entry in entries {
        let snapshot = FastSyncSnapshotCatalogEntry::try_from(entry)?;
        validate_snapshot_catalog_entry(&snapshot)?;
        if !ids.insert(snapshot.id.clone()) {
            anyhow::bail!("duplicate snapshot catalog id: {}", snapshot.id);
        }
        snapshots.push(snapshot);
    }
    Ok(snapshots)
}

impl TryFrom<SnapshotCatalogEntryDto> for FastSyncSnapshotCatalogEntry {
    type Error = anyhow::Error;

    fn try_from(value: SnapshotCatalogEntryDto) -> Result<Self> {
        let snapshot_id = value.id.trim().to_string();
        let node_type = NodeType::from_str(value.node_type.trim())
            .with_context(|| format!("snapshot catalog {snapshot_id} has invalid runtime"))?;
        let network = Network::from_str(value.network.trim())
            .with_context(|| format!("snapshot catalog {snapshot_id} has invalid network"))?;
        let url = value.url.trim().to_string();
        let file_name = catalog_file_name(value.file_name, &url);

        Ok(Self {
            id: snapshot_id,
            label: value.label.trim().to_string(),
            network,
            node_type,
            url,
            file_name,
            expected_sha256: value.expected_sha256.trim().to_string(),
            max_bytes: value
                .max_bytes
                .unwrap_or(FastSyncSnapshotManager::DEFAULT_DOWNLOAD_MAX_BYTES),
        })
    }
}

fn catalog_file_name(file_name: Option<String>, url: &str) -> String {
    file_name
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| file_name_from_url(url).unwrap_or_else(|_| "snapshot.acc".to_string()))
}
