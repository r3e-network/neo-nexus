use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub(in crate::runtime) fn verified_source_path(path: &Path) -> Result<PathBuf> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("runtime package source {} was not found", path.display()))?;
    if !canonical.is_file() {
        anyhow::bail!(
            "runtime package source {} is not a file",
            canonical.display()
        );
    }
    Ok(canonical)
}
