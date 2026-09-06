use std::{fs::File, io::Read, path::Path};

use anyhow::{Context, Result};

pub(in crate::release_pack) fn validate_checksum_file(
    path: &Path,
    expected_file_name: &str,
    expected_sha256: &str,
) -> Result<()> {
    const MAX_CHECKSUM_BYTES: u64 = 4 * 1024;
    let mut file = File::open(path)
        .with_context(|| format!("failed to open release checksum {}", path.display()))?;
    let mut bytes = Vec::new();
    file.by_ref()
        .take(MAX_CHECKSUM_BYTES + 1)
        .read_to_end(&mut bytes)
        .with_context(|| format!("failed to read release checksum {}", path.display()))?;
    if bytes.len() as u64 > MAX_CHECKSUM_BYTES {
        anyhow::bail!("release checksum exceeds {MAX_CHECKSUM_BYTES} byte verification limit");
    }
    let text = std::str::from_utf8(&bytes).context("release checksum must be UTF-8")?;
    let mut parts = text.split_whitespace();
    let Some(actual_sha256) = parts.next() else {
        anyhow::bail!("release checksum {} is empty", path.display());
    };
    let actual_file_name = parts.next().unwrap_or("");
    if actual_sha256 != expected_sha256 {
        anyhow::bail!(
            "release checksum SHA-256 mismatch: expected {}, got {}",
            expected_sha256,
            actual_sha256
        );
    }
    if actual_file_name != expected_file_name {
        anyhow::bail!(
            "release checksum filename mismatch: expected {}, got {}",
            expected_file_name,
            actual_file_name
        );
    }
    if parts.next().is_some() {
        anyhow::bail!("release checksum contains unexpected trailing fields");
    }
    Ok(())
}
