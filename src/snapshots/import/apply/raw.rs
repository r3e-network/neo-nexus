use std::{fs, path::Path};

use anyhow::Result;

use crate::snapshots::{
    io::{copy_file_hashed, replace_file},
    normalize_sha256, FastSyncSnapshot,
};

use super::super::{archive::SnapshotImport, mode::applied_filename};

pub(super) fn import_raw_snapshot(
    snapshot: &FastSyncSnapshot,
    cached_path: &Path,
    control_dir: &Path,
) -> Result<SnapshotImport> {
    let snapshot_path = control_dir.join(applied_filename(snapshot, cached_path));
    let temp_path = snapshot_path.with_extension("apply");
    let (copied_sha256, copied_bytes) = copy_file_hashed(cached_path, &temp_path)?;
    let expected_sha256 = normalize_sha256(&snapshot.expected_sha256)?;
    if copied_sha256 != expected_sha256 {
        let _ = fs::remove_file(&temp_path);
        anyhow::bail!("snapshot copy verification failed before applying to node data dir");
    }
    replace_file(&temp_path, &snapshot_path)?;
    Ok(SnapshotImport {
        snapshot_path,
        imported_files: 1,
        expanded_bytes: copied_bytes,
    })
}
