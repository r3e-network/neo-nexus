use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use zip::ZipArchive;

mod copy;
mod tracker;

use self::{copy::copy_archive_file, tracker::ArchiveImportTracker};
use super::paths::{create_safe_directory, prepare_new_archive_file, safe_archive_relative_path};

pub(super) struct SnapshotImport {
    pub(super) snapshot_path: PathBuf,
    pub(super) imported_files: usize,
    pub(super) expanded_bytes: u64,
}

pub(super) fn unpack_tar_snapshot(source: &Path, target_dir: &Path) -> Result<SnapshotImport> {
    let file = File::open(source)
        .with_context(|| format!("failed to open archive {}", source.display()))?;
    let mut archive = tar::Archive::new(file);
    unpack_tar_entries(&mut archive, target_dir)
}

pub(super) fn unpack_tar_gzip_snapshot(source: &Path, target_dir: &Path) -> Result<SnapshotImport> {
    let file = File::open(source)
        .with_context(|| format!("failed to open archive {}", source.display()))?;
    let decoder = GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    unpack_tar_entries(&mut archive, target_dir)
}

pub(super) fn unpack_zip_snapshot(source: &Path, target_dir: &Path) -> Result<SnapshotImport> {
    let file = File::open(source)
        .with_context(|| format!("failed to open archive {}", source.display()))?;
    let mut archive = ZipArchive::new(file).context("failed to read zip archive")?;
    let mut tracker = ArchiveImportTracker::new();

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .context("failed to read zip archive entry")?;
        if entry
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            anyhow::bail!("snapshot zip archive contains a symbolic link entry");
        }
        let enclosed_name = entry
            .enclosed_name()
            .context("snapshot zip archive entry path is unsafe")?;
        let Some(relative_path) = safe_archive_relative_path(&enclosed_name)? else {
            continue;
        };

        if entry.is_dir() {
            create_safe_directory(target_dir, &relative_path)?;
        } else if entry.is_file() {
            let target_path = prepare_new_archive_file(target_dir, &relative_path)?;
            copy_archive_file(&mut entry, &target_path, &mut tracker)?;
        } else {
            anyhow::bail!(
                "snapshot zip archive entry {} has unsupported type",
                relative_path.display()
            );
        }
    }

    Ok(tracker.finish(target_dir))
}

fn unpack_tar_entries<R: Read>(
    archive: &mut tar::Archive<R>,
    target_dir: &Path,
) -> Result<SnapshotImport> {
    let mut tracker = ArchiveImportTracker::new();
    let entries = archive.entries().context("failed to read tar archive")?;
    for entry in entries {
        let mut entry = entry.context("failed to read tar archive entry")?;
        let entry_path = entry
            .path()
            .context("failed to read tar archive entry path")?
            .into_owned();
        let Some(relative_path) = safe_archive_relative_path(&entry_path)? else {
            continue;
        };

        let entry_type = entry.header().entry_type();
        if entry_type.is_dir() {
            create_safe_directory(target_dir, &relative_path)?;
        } else if entry_type.is_file() {
            let target_path = prepare_new_archive_file(target_dir, &relative_path)?;
            copy_archive_file(&mut entry, &target_path, &mut tracker)?;
        } else {
            anyhow::bail!(
                "snapshot archive entry {} has unsupported type",
                relative_path.display()
            );
        }
    }

    Ok(tracker.finish(target_dir))
}
