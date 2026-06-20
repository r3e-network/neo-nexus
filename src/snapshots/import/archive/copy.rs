use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

use anyhow::{Context, Result};

use super::tracker::ArchiveImportTracker;
use crate::snapshots::import::paths::temporary_import_path;

pub(super) fn copy_archive_file(
    reader: &mut impl Read,
    target: &Path,
    tracker: &mut ArchiveImportTracker,
) -> Result<()> {
    tracker.reserve_file()?;
    let temp_path = temporary_import_path(target);
    if temp_path.exists() {
        fs::remove_file(&temp_path).with_context(|| {
            format!(
                "failed to remove stale snapshot import file {}",
                temp_path.display()
            )
        })?;
    }
    let mut output = File::create(&temp_path)
        .with_context(|| format!("failed to create snapshot import {}", temp_path.display()))?;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = reader
            .read(&mut buffer)
            .context("failed to read snapshot archive entry")?;
        if read == 0 {
            break;
        }
        tracker.reserve_bytes(read as u64)?;
        output
            .write_all(&buffer[..read])
            .with_context(|| format!("failed to write snapshot import {}", temp_path.display()))?;
    }
    output
        .sync_all()
        .with_context(|| format!("failed to sync snapshot import {}", temp_path.display()))?;
    if target.exists() {
        let _ = fs::remove_file(&temp_path);
        anyhow::bail!("snapshot import target {} already exists", target.display());
    }
    fs::rename(&temp_path, target).with_context(|| {
        format!(
            "failed to publish snapshot import {} to {}",
            temp_path.display(),
            target.display()
        )
    })
}
