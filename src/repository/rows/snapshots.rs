use std::path::PathBuf;

use crate::{
    snapshots::{FastSyncSnapshot, FastSyncSnapshotManager},
    types::{Network, NodeType},
};

use super::parse_field;

pub(in crate::repository) fn snapshot_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<FastSyncSnapshot> {
    let network_raw: String = row.get(2)?;
    let node_type_raw: String = row.get(3)?;
    let cached_path: Option<String> = row.get(9)?;

    Ok(FastSyncSnapshot {
        id: row.get(0)?,
        label: row.get(1)?,
        network: parse_field::<Network>(&network_raw)?,
        node_type: parse_field::<NodeType>(&node_type_raw)?,
        source_path: PathBuf::from(row.get::<_, String>(4)?),
        source_url: row.get(5)?,
        download_file_name: row.get(6)?,
        download_max_bytes: row
            .get::<_, Option<u64>>(7)?
            .unwrap_or(FastSyncSnapshotManager::DEFAULT_DOWNLOAD_MAX_BYTES),
        expected_sha256: row.get(8)?,
        cached_path: cached_path.map(PathBuf::from),
        verified_sha256: row.get(10)?,
        verified_at_unix: row.get(11)?,
        bytes: row.get(12)?,
    })
}
