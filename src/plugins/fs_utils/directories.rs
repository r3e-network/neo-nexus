use std::{fs, io::ErrorKind, path::Path};

use anyhow::{Context, Result};

pub(in crate::plugins) fn ensure_real_directory_exists(path: &Path, label: &str) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                anyhow::bail!("{label} {} must be a real directory", path.display());
            }
            Ok(())
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            fs::create_dir_all(path)
                .with_context(|| format!("failed to create {label} {}", path.display()))?;
            let metadata = fs::symlink_metadata(path)
                .with_context(|| format!("failed to inspect {label} {}", path.display()))?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                anyhow::bail!("{label} {} must be a real directory", path.display());
            }
            Ok(())
        }
        Err(error) => {
            Err(error).with_context(|| format!("failed to inspect {label} {}", path.display()))
        }
    }
}

pub(in crate::plugins) fn reset_directory(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                anyhow::bail!("plugin staging path {} is not a directory", path.display());
            }
            fs::remove_dir_all(path)
                .with_context(|| format!("failed to reset plugin staging {}", path.display()))?;
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error)
                .with_context(|| format!("failed to inspect plugin staging {}", path.display()));
        }
    }
    fs::create_dir_all(path)
        .with_context(|| format!("failed to create plugin staging {}", path.display()))
}
