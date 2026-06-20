use std::{fs, io::ErrorKind, path::Path};

use anyhow::{Context, Result};

use super::directories::ensure_real_directory_exists;

pub(in crate::plugins) fn replace_plugin_directory(
    staging_dir: &Path,
    target_dir: &Path,
    backup_dir: &Path,
) -> Result<()> {
    let mut moved_existing = false;
    if let Some(parent) = backup_dir.parent() {
        ensure_real_directory_exists(parent, "plugin backup directory")?;
    }

    match fs::symlink_metadata(target_dir) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                anyhow::bail!(
                    "plugin target {} is not a real directory",
                    target_dir.display()
                );
            }
            fs::rename(target_dir, backup_dir).with_context(|| {
                format!(
                    "failed to move existing plugin {} to backup {}",
                    target_dir.display(),
                    backup_dir.display()
                )
            })?;
            moved_existing = true;
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error).with_context(|| {
                format!("failed to inspect plugin target {}", target_dir.display())
            });
        }
    }

    if let Err(error) = fs::rename(staging_dir, target_dir) {
        if moved_existing {
            let _ = fs::rename(backup_dir, target_dir);
        }
        return Err(error).with_context(|| {
            format!(
                "failed to publish plugin package {} to {}",
                staging_dir.display(),
                target_dir.display()
            )
        });
    }

    if moved_existing {
        fs::remove_dir_all(backup_dir)
            .with_context(|| format!("failed to remove plugin backup {}", backup_dir.display()))?;
    }
    Ok(())
}
