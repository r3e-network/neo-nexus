use std::{
    fs,
    io::Write,
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
    // Preserve repeated exports in the same second. Publish a complete file so
    // backup discovery cannot select a partially written JSON document.
    let name = format!(
        "neonexus-backup-{exported_at_unix}-{}",
        uuid::Uuid::new_v4()
    );
    let path = output_dir.join(format!("{name}.json"));
    let temporary = output_dir.join(format!(".{name}.tmp"));
    let result = (|| -> Result<()> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, &path)?;
        Ok(())
    })();
    if let Err(error) = result {
        let _ = fs::remove_file(&temporary);
        return Err(error).with_context(|| format!("failed to write backup {}", path.display()));
    }

    Ok(WrittenBackup {
        path,
        bytes_written: text.len(),
    })
}
