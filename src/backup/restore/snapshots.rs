use std::{path::PathBuf, str::FromStr};

use anyhow::{Context, Result};

use crate::{
    snapshots::{validate_snapshot_input, NewFastSyncSnapshot},
    types::{Network, NodeType},
};

use super::super::schema::FastSyncSnapshotBackup;

pub(in crate::backup) fn restored_fast_sync_snapshot(
    backup: &FastSyncSnapshotBackup,
) -> Result<NewFastSyncSnapshot> {
    let snapshot = NewFastSyncSnapshot {
        id: backup.id.clone(),
        label: backup.label.clone(),
        network: Network::from_str(&backup.network)
            .with_context(|| format!("backup snapshot {} has invalid network", backup.id))?,
        node_type: NodeType::from_str(&backup.node_type)
            .with_context(|| format!("backup snapshot {} has invalid runtime", backup.id))?,
        source_path: PathBuf::from(&backup.source_path),
        source_url: backup.source_url.clone(),
        download_file_name: backup.download_file_name.clone(),
        download_max_bytes: backup.download_max_bytes,
        expected_sha256: backup.expected_sha256.clone(),
    };
    validate_snapshot_input(&snapshot)?;
    Ok(snapshot)
}
