use std::path::Path;

use anyhow::Result;

use crate::snapshots::FastSyncSnapshot;

use super::{archive_payload, raw};
use crate::snapshots::import::{archive::SnapshotImport, SnapshotImportMode};

pub(super) fn import_cached_snapshot(
    snapshot: &FastSyncSnapshot,
    cached_path: &Path,
    import_mode: SnapshotImportMode,
    control_dir: &Path,
    import_dir: &Path,
) -> Result<SnapshotImport> {
    match import_mode {
        SnapshotImportMode::RawFile => raw::import_raw_snapshot(snapshot, cached_path, control_dir),
        SnapshotImportMode::TarArchive
        | SnapshotImportMode::TarGzipArchive
        | SnapshotImportMode::ZipArchive => archive_payload::import_archive_snapshot(
            cached_path,
            import_mode,
            control_dir,
            import_dir,
        ),
    }
}
