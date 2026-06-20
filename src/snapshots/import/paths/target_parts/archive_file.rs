use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use super::directory::{create_safe_directory, validate_safe_target_directory};

pub(in crate::snapshots::import) fn validate_new_archive_file(
    root: &Path,
    relative: &Path,
) -> Result<()> {
    if let Some(parent) = non_empty_parent(relative) {
        validate_safe_target_directory(root, parent)?;
    }
    ensure_archive_target_is_new(&root.join(relative)).map(|_| ())
}

pub(in crate::snapshots::import) fn prepare_new_archive_file(
    root: &Path,
    relative: &Path,
) -> Result<PathBuf> {
    if let Some(parent) = non_empty_parent(relative) {
        create_safe_directory(root, parent)?;
    }
    ensure_archive_target_is_new(&root.join(relative))
}

fn non_empty_parent(relative: &Path) -> Option<&Path> {
    relative
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
}

fn ensure_archive_target_is_new(target: &Path) -> Result<PathBuf> {
    match fs::symlink_metadata(target) {
        Ok(_) => anyhow::bail!("snapshot import target {} already exists", target.display()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(target.to_path_buf()),
        Err(error) => Err(error)
            .with_context(|| format!("failed to inspect import target {}", target.display())),
    }
}
