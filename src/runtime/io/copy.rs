use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

pub(in crate::runtime) fn copy_file_hashed(source: &Path, target: &Path) -> Result<(String, u64)> {
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

pub(in crate::runtime) fn copy_reader_hashed(
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
            .context("failed to read runtime download stream")?;
        if read == 0 {
            break;
        }
        bytes += read as u64;
        if bytes > max_bytes {
            let _ = fs::remove_file(target);
            anyhow::bail!(
                "runtime download exceeded size limit: {bytes} bytes exceeds limit {max_bytes}"
            );
        }
        output
            .write_all(&buffer[..read])
            .with_context(|| format!("failed to write file {}", target.display()))?;
        hasher.update(&buffer[..read]);
    }

    output
        .sync_all()
        .with_context(|| format!("failed to sync file {}", target.display()))?;
    Ok((hex_lower(&hasher.finalize()), bytes))
}

pub(in crate::runtime) fn replace_file(source: &Path, target: &Path) -> Result<()> {
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

fn hex_lower(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(hex_digit(byte >> 4));
        output.push(hex_digit(byte & 0x0f));
    }
    output
}

fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'a' + value - 10) as char,
        _ => '0',
    }
}
