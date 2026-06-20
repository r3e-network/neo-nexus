use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub(in crate::snapshots) fn verified_source_path(path: &Path) -> Result<PathBuf> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("snapshot source {} was not found", path.display()))?;
    if !canonical.is_file() {
        anyhow::bail!("snapshot source {} is not a file", canonical.display());
    }
    Ok(canonical)
}
