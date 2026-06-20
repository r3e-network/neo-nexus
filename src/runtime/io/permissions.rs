use std::path::Path;

use anyhow::{Context, Result};

#[cfg(unix)]
pub(in crate::runtime) fn make_executable(path: &Path) -> Result<()> {
    use std::{fs, os::unix::fs::PermissionsExt};

    let mut permissions = fs::metadata(path)
        .with_context(|| format!("failed to inspect runtime binary {}", path.display()))?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).with_context(|| {
        format!(
            "failed to mark runtime binary executable {}",
            path.display()
        )
    })
}

#[cfg(not(unix))]
pub(in crate::runtime) fn make_executable(path: &Path) -> Result<()> {
    if path.is_file() {
        Ok(())
    } else {
        anyhow::bail!("runtime binary {} was not installed", path.display());
    }
}
