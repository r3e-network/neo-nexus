use crate::snapshots::FastSyncSnapshot;

use super::super::schema::FastSyncSnapshotBackup;

pub(in crate::backup) fn fast_sync_snapshot_backup(
    snapshot: FastSyncSnapshot,
) -> FastSyncSnapshotBackup {
    FastSyncSnapshotBackup {
        id: snapshot.id,
        label: snapshot.label,
        network: snapshot.network.to_string(),
        node_type: snapshot.node_type.to_string(),
        source_path: snapshot.source_path.display().to_string(),
        source_url: snapshot.source_url,
        download_file_name: snapshot.download_file_name,
        download_max_bytes: snapshot.download_max_bytes,
        expected_sha256: snapshot.expected_sha256,
    }
}
