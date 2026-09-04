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

    let mut latest: Option<(u64, std::time::SystemTime, PathBuf)> = None;
    for entry in fs::read_dir(input_dir)
        .with_context(|| format!("failed to read backup directory {}", input_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let Some(timestamp) = backup_timestamp_from_path(&path) else {
            continue;
        };
        let metadata = entry.metadata()?;
        if !metadata.is_file() {
            continue;
        }
        let modified = metadata.modified().unwrap_or(std::time::UNIX_EPOCH);
        if latest
            .as_ref()
            .is_none_or(|(latest_timestamp, latest_modified, latest_path)| {
                (timestamp, modified, &path) > (*latest_timestamp, *latest_modified, latest_path)
            })
        {
            latest = Some((timestamp, modified, path));
        }
    }

    Ok(latest.map(|(_, _, path)| path))
}

fn backup_timestamp_from_path(path: &Path) -> Option<u64> {
    let file_name = path.file_name()?.to_str()?;
    let timestamp = file_name
        .strip_prefix("neonexus-backup-")?
        .strip_suffix(".json")?;
    if let Some((timestamp, suffix)) = timestamp.split_once('-') {
        uuid::Uuid::parse_str(suffix).ok()?;
        timestamp.parse::<u64>().ok()
    } else {
        timestamp.parse::<u64>().ok()
    }
}
