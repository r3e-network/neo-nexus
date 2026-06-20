use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

pub(super) fn latest_backup_path(input_dir: impl AsRef<Path>) -> Result<Option<PathBuf>> {
    let input_dir = input_dir.as_ref();
    if !input_dir.is_dir() {
        return Ok(None);
    }

    let mut latest: Option<(u64, PathBuf)> = None;
    for entry in fs::read_dir(input_dir)
        .with_context(|| format!("failed to read backup directory {}", input_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let Some(timestamp) = backup_timestamp_from_path(&path) else {
            continue;
        };
        if latest
            .as_ref()
            .is_none_or(|(latest_timestamp, _)| timestamp > *latest_timestamp)
        {
            latest = Some((timestamp, path));
        }
    }

    Ok(latest.map(|(_, path)| path))
}

fn backup_timestamp_from_path(path: &Path) -> Option<u64> {
    let file_name = path.file_name()?.to_str()?;
    let timestamp = file_name
        .strip_prefix("neonexus-backup-")?
        .strip_suffix(".json")?;
    timestamp.parse::<u64>().ok()
}
