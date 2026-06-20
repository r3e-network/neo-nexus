use std::{fs, path::Path};

use anyhow::{Context, Result};

use super::paths::{
    create_safe_directory, prepare_new_archive_file, validate_new_archive_file,
    validate_safe_target_directory,
};

pub(super) fn publish_staged_import(staging_dir: &Path, import_dir: &Path) -> Result<()> {
    preflight_staged_import(staging_dir, staging_dir, import_dir)?;
    publish_staged_import_entries(staging_dir, staging_dir, import_dir)
}

fn preflight_staged_import(root: &Path, current: &Path, import_dir: &Path) -> Result<()> {
    for entry in fs::read_dir(current)
        .with_context(|| format!("failed to read staged snapshot {}", current.display()))?
    {
        let entry = entry.context("failed to read staged snapshot entry")?;
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .context("staged snapshot path escaped its root")?;
        let metadata = fs::symlink_metadata(&path)
            .with_context(|| format!("failed to inspect staged snapshot {}", path.display()))?;
        if metadata.file_type().is_symlink() {
            anyhow::bail!("staged snapshot contains a symbolic link");
        }
        if metadata.is_dir() {
            validate_safe_target_directory(import_dir, relative)?;
            preflight_staged_import(root, &path, import_dir)?;
        } else if metadata.is_file() {
            validate_new_archive_file(import_dir, relative)?;
        } else {
            anyhow::bail!(
                "staged snapshot entry {} is not a regular file",
                relative.display()
            );
        }
    }
    Ok(())
}

fn publish_staged_import_entries(root: &Path, current: &Path, import_dir: &Path) -> Result<()> {
    for entry in fs::read_dir(current)
        .with_context(|| format!("failed to read staged snapshot {}", current.display()))?
    {
        let entry = entry.context("failed to read staged snapshot entry")?;
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .context("staged snapshot path escaped its root")?;
        let metadata = fs::symlink_metadata(&path)
            .with_context(|| format!("failed to inspect staged snapshot {}", path.display()))?;
        if metadata.is_dir() {
            create_safe_directory(import_dir, relative)?;
            publish_staged_import_entries(root, &path, import_dir)?;
        } else if metadata.is_file() {
            let target = prepare_new_archive_file(import_dir, relative)?;
            fs::rename(&path, &target).with_context(|| {
                format!(
                    "failed to move staged snapshot {} to {}",
                    path.display(),
                    target.display()
                )
            })?;
        }
    }
    Ok(())
}
