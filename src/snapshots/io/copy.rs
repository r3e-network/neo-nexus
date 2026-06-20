use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

use super::hash::hex_lower;

pub(in crate::snapshots) fn copy_file_hashed(
    source: &Path,
    target: &Path,
) -> Result<(String, u64)> {
    let mut input =
        File::open(source).with_context(|| format!("failed to open file {}", source.display()))?;
    let mut output = File::create(target)
        .with_context(|| format!("failed to create file {}", target.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    let mut bytes = 0u64;

    loop {
        let read = input
            .read(&mut buffer)
            .with_context(|| format!("failed to read file {}", source.display()))?;
        if read == 0 {
            break;
        }
        output
            .write_all(&buffer[..read])
            .with_context(|| format!("failed to write file {}", target.display()))?;
        hasher.update(&buffer[..read]);
        bytes += read as u64;
    }

    output
        .sync_all()
        .with_context(|| format!("failed to sync file {}", target.display()))?;
    Ok((hex_lower(&hasher.finalize()), bytes))
}

pub(in crate::snapshots) fn copy_reader_hashed(
    mut reader: impl Read,
    target: &Path,
    max_bytes: u64,
) -> Result<(String, u64)> {
    let mut output = File::create(target)
        .with_context(|| format!("failed to create file {}", target.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    let mut bytes = 0u64;

    loop {
        let read = reader
            .read(&mut buffer)
            .context("failed to read snapshot download")?;
        if read == 0 {
            break;
        }
        let next_bytes = bytes.saturating_add(read as u64);
        if next_bytes > max_bytes {
            let _ = fs::remove_file(target);
            anyhow::bail!("snapshot download exceeded size limit {max_bytes} bytes");
        }
        output
            .write_all(&buffer[..read])
            .with_context(|| format!("failed to write file {}", target.display()))?;
        hasher.update(&buffer[..read]);
        bytes = next_bytes;
    }

    output
        .sync_all()
        .with_context(|| format!("failed to sync file {}", target.display()))?;
    Ok((hex_lower(&hasher.finalize()), bytes))
}
