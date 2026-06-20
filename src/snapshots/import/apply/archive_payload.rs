use std::{fs, path::Path};

use anyhow::Result;

use super::super::{
    archive::{unpack_tar_gzip_snapshot, unpack_tar_snapshot, unpack_zip_snapshot, SnapshotImport},
    paths::reset_directory,
    publish::publish_staged_import,
    SnapshotImportMode,
};

pub(super) fn import_archive_snapshot(
    cached_path: &Path,
    import_mode: SnapshotImportMode,
    control_dir: &Path,
    import_dir: &Path,
) -> Result<SnapshotImport> {
    let staging_dir = control_dir.join("payload.staging");
    reset_directory(&staging_dir)?;
    let import_result = match import_mode {
        SnapshotImportMode::TarArchive => unpack_tar_snapshot(cached_path, &staging_dir),
        SnapshotImportMode::TarGzipArchive => unpack_tar_gzip_snapshot(cached_path, &staging_dir),
        SnapshotImportMode::ZipArchive => unpack_zip_snapshot(cached_path, &staging_dir),
        SnapshotImportMode::RawFile => anyhow::bail!("raw snapshots are not archives"),
    };

    let import = match import_result {
        Ok(import) => import,
        Err(error) => {
            let _ = fs::remove_dir_all(&staging_dir);
            return Err(error);
        }
    };

    if import.imported_files == 0 {
        let _ = fs::remove_dir_all(&staging_dir);
        anyhow::bail!("snapshot archive did not contain importable files");
    }

    if let Err(error) = publish_staged_import(&staging_dir, import_dir) {
        let _ = fs::remove_dir_all(&staging_dir);
        return Err(error);
    }
    let _ = fs::remove_dir_all(&staging_dir);

    Ok(SnapshotImport {
        snapshot_path: import_dir.to_path_buf(),
        imported_files: import.imported_files,
        expanded_bytes: import.expanded_bytes,
    })
}
