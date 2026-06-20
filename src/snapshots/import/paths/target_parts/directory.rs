use std::{
    fs,
    io::ErrorKind,
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result};

pub(in crate::snapshots::import) fn create_safe_directory(
    root: &Path,
    relative: &Path,
) -> Result<PathBuf> {
    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(safe_directory_component(relative, component)?);
        ensure_safe_target_directory_exists(&current)?;
    }
    Ok(current)
}

pub(in crate::snapshots::import) fn validate_safe_target_directory(
    root: &Path,
    relative: &Path,
) -> Result<()> {
    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(safe_directory_component(relative, component)?);
        inspect_existing_or_missing_directory(&current)?;
    }
    Ok(())
}

fn safe_directory_component<'a>(
    relative: &Path,
    component: Component<'a>,
) -> Result<&'a std::ffi::OsStr> {
    let Component::Normal(part) = component else {
        anyhow::bail!("snapshot import directory {} is unsafe", relative.display());
    };
    Ok(part)
}

fn inspect_existing_or_missing_directory(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => ensure_target_is_directory(path, &metadata),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error)
            .with_context(|| format!("failed to inspect import target {}", path.display())),
    }
}

fn ensure_safe_target_directory_exists(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => ensure_target_is_directory(path, &metadata),
        Err(error) if error.kind() == ErrorKind::NotFound => {
            fs::create_dir(path).with_context(|| {
                format!(
                    "failed to create snapshot import directory {}",
                    path.display()
                )
            })
        }
        Err(error) => Err(error)
            .with_context(|| format!("failed to inspect import target {}", path.display())),
    }
}

fn ensure_target_is_directory(path: &Path, metadata: &fs::Metadata) -> Result<()> {
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        anyhow::bail!(
            "snapshot import target {} is not a directory",
            path.display()
        );
    }
    Ok(())
}
