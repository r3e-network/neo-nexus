use std::path::Path;

use anyhow::Result;

use crate::types::NodeConfig;

mod archive_payload;
mod dispatch;
mod prepare;
mod raw;
mod validate;

use super::super::{current_unix_time, FastSyncSnapshot};
use super::{
    manifest::{write_applied_snapshot_manifest, AppliedSnapshotManifestInput},
    mode::snapshot_import_mode,
    SnapshotApplication,
};
use dispatch::import_cached_snapshot;
use prepare::prepare_snapshot_import_dirs;
use validate::{verified_cached_snapshot, verify_snapshot_matches_node};

pub(in crate::snapshots) fn apply_to_node(
    snapshot: &FastSyncSnapshot,
    node: &NodeConfig,
    node_data_dir: &Path,
) -> Result<SnapshotApplication> {
    verify_snapshot_matches_node(snapshot, node)?;
    let cached = verified_cached_snapshot(snapshot)?;

    let applied_at_unix = current_unix_time()?;
    let dirs = prepare_snapshot_import_dirs(node_data_dir, snapshot)?;
    let import_mode = snapshot_import_mode(snapshot, &cached.path);
    let import = import_cached_snapshot(
        snapshot,
        &cached.path,
        import_mode,
        &dirs.control_dir,
        &dirs.import_dir,
    )?;

    let manifest_path = dirs.control_dir.join("manifest.json");
    write_applied_snapshot_manifest(
        &manifest_path,
        AppliedSnapshotManifestInput {
            snapshot_id: &snapshot.id,
            label: &snapshot.label,
            node,
            import_mode,
            sha256: &cached.sha256,
            bytes: cached.bytes,
            cached_path: &cached.path,
            import_dir: &dirs.import_dir,
            import: &import,
            applied_at_unix,
        },
    )?;

    Ok(SnapshotApplication {
        snapshot_path: import.snapshot_path,
        import_dir: dirs.import_dir,
        manifest_path,
        import_mode,
        imported_files: import.imported_files,
        expanded_bytes: import.expanded_bytes,
        sha256: cached.sha256,
        bytes: cached.bytes,
        applied_at_unix,
    })
}
