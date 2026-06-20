use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use super::WorkspaceBackup;

pub(super) struct WrittenBackup {
    pub(super) path: PathBuf,
    pub(super) bytes_written: usize,
}

pub(super) fn write_backup_file(
    backup: &WorkspaceBackup,
    output_dir: &Path,
    exported_at_unix: u64,
) -> Result<WrittenBackup> {
    let text = serde_json::to_string_pretty(backup).context("failed to render workspace backup")?;
    fs::create_dir_all(output_dir)
        .with_context(|| format!("failed to create backup directory {}", output_dir.display()))?;
    let path = output_dir.join(format!("neonexus-backup-{exported_at_unix}.json"));
    fs::write(&path, text.as_bytes())
        .with_context(|| format!("failed to write backup {}", path.display()))?;

    Ok(WrittenBackup {
        path,
        bytes_written: text.len(),
    })
}
