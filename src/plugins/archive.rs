use std::{fs::File, path::Path};

use anyhow::{Context, Result};
use zip::ZipArchive;

mod copy;
mod paths;
mod tracker;

use self::{
    copy::copy_archive_file,
    paths::{create_safe_directory, prepare_new_archive_file, safe_archive_relative_path},
    tracker::ZipInstallTracker,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ZipInstallResult {
    pub(super) installed_files: usize,
    pub(super) expanded_bytes: u64,
}

pub(super) fn unpack_plugin_zip(source: &Path, target_dir: &Path) -> Result<ZipInstallResult> {
    let file = File::open(source)
        .with_context(|| format!("failed to open plugin package {}", source.display()))?;
    let mut archive = ZipArchive::new(file).context("failed to read plugin zip package")?;
    let mut tracker = ZipInstallTracker::new();

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .context("failed to read plugin zip package entry")?;
        if entry
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            anyhow::bail!("plugin zip package contains a symbolic link entry");
        }
        let enclosed_name = entry
            .enclosed_name()
            .context("plugin zip package entry path is unsafe")?;
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
                "plugin zip package entry {} has unsupported type",
                relative_path.display()
            );
        }
    }

    Ok(tracker.finish())
}
