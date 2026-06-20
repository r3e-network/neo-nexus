use std::{fs, io::ErrorKind, path::Path};

use anyhow::{Context, Result};

pub(in crate::snapshots::import) const SNAPSHOT_CONTROL_DIR: &str = "fastsync";

pub(in crate::snapshots::import) fn ensure_import_root(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .with_context(|| format!("failed to create snapshot import root {}", path.display()))?;
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("failed to inspect snapshot import root {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        anyhow::bail!(
            "snapshot import root {} must be a real directory",
            path.display()
        );
    }
    Ok(())
}

pub(in crate::snapshots::import) fn reset_directory(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                anyhow::bail!(
                    "snapshot staging path {} is not a directory",
                    path.display()
                );
            }
            fs::remove_dir_all(path)
                .with_context(|| format!("failed to reset snapshot staging {}", path.display()))?;
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error)
                .with_context(|| format!("failed to inspect snapshot staging {}", path.display()));
        }
    }
    fs::create_dir_all(path)
        .with_context(|| format!("failed to create snapshot staging {}", path.display()))
}
