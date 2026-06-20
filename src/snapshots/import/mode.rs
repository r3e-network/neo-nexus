use std::path::Path;

use crate::snapshots::FastSyncSnapshot;

use super::super::cache::{safe_fragment, snapshot_file_extension};
use super::SnapshotImportMode;

pub(super) fn snapshot_import_mode(
    snapshot: &FastSyncSnapshot,
    cached_path: &Path,
) -> SnapshotImportMode {
    snapshot
        .download_file_name
        .as_deref()
        .and_then(import_mode_from_file_name)
        .or_else(|| {
            snapshot
                .source_path
                .file_name()
                .and_then(|value| value.to_str())
                .and_then(import_mode_from_file_name)
        })
        .or_else(|| {
            cached_path
                .file_name()
                .and_then(|value| value.to_str())
                .and_then(import_mode_from_file_name)
        })
        .unwrap_or(SnapshotImportMode::RawFile)
}

pub(super) fn applied_filename(snapshot: &FastSyncSnapshot, cached_path: &Path) -> String {
    let extension = cached_path
        .file_name()
        .and_then(|value| value.to_str())
        .and_then(snapshot_file_extension)
        .map(safe_fragment)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "snapshot".to_string());
    format!("{}.{extension}", safe_fragment(&snapshot.id))
}

fn import_mode_from_file_name(file_name: &str) -> Option<SnapshotImportMode> {
    let name = file_name.trim().to_ascii_lowercase();
    if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        Some(SnapshotImportMode::TarGzipArchive)
    } else if name.ends_with(".tar") {
        Some(SnapshotImportMode::TarArchive)
    } else if name.ends_with(".zip") {
        Some(SnapshotImportMode::ZipArchive)
    } else {
        None
    }
}
