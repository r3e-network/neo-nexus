use std::{fs, path::Path};

use anyhow::{Context, Result};

pub(in crate::release_pack) fn validate_checksum_file(
    path: &Path,
    expected_file_name: &str,
    expected_sha256: &str,
) -> Result<()> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read release checksum {}", path.display()))?;
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
    Ok(())
}
