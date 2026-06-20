use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use anyhow::{Context, Result};

use super::tracker::ZipInstallTracker;

pub(super) fn copy_archive_file(
    reader: &mut impl Read,
    target: &Path,
    tracker: &mut ZipInstallTracker,
) -> Result<()> {
    tracker.reserve_file()?;
    let mut output = File::create(target)
        .with_context(|| format!("failed to create plugin file {}", target.display()))?;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = reader
            .read(&mut buffer)
            .context("failed to read plugin zip package entry")?;
        if read == 0 {
            break;
        }
        tracker.reserve_bytes(read as u64)?;
        output
            .write_all(&buffer[..read])
            .with_context(|| format!("failed to write plugin file {}", target.display()))?;
    }
    output
        .sync_all()
        .with_context(|| format!("failed to sync plugin file {}", target.display()))
}
