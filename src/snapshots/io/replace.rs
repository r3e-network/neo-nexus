use std::{fs, path::Path};

use anyhow::{Context, Result};

pub(in crate::snapshots) fn replace_file(source: &Path, target: &Path) -> Result<()> {
    if target.exists() {
        fs::remove_file(target)
            .with_context(|| format!("failed to replace file {}", target.display()))?;
    }
    fs::rename(source, target).with_context(|| {
        format!(
            "failed to move file {} to {}",
            source.display(),
            target.display()
        )
    })
}
