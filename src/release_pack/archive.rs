use std::{
    collections::HashSet,
    fs::File,
    io::{Read, Write},
    path::Path,
};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

use super::{
    manifest::ReleaseArchiveManifestOwned,
    validation::{safe_file_name, MAX_RELEASE_BINARY_BYTES},
};

const EXPECTED_ARCHIVE_ENTRIES: usize = 2;
const MAX_ARCHIVE_MANIFEST_BYTES: u64 = 64 * 1024;
const MAX_ARCHIVE_EXPANDED_BYTES: u64 = MAX_RELEASE_BINARY_BYTES + MAX_ARCHIVE_MANIFEST_BYTES;
const MAX_COMPRESSION_RATIO: u64 = 200;

pub(super) fn write_release_archive(
    archive_path: &Path,
    binary_path: &Path,
    binary_name: &str,
    manifest_text: &str,
) -> Result<()> {
    let file = File::create(archive_path).with_context(|| {
        format!(
            "failed to create release archive {}",
            archive_path.display()
        )
    })?;
    let mut zip = ZipWriter::new(file);
    let binary_options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);
    zip.start_file(binary_name, binary_options)
        .context("failed to start binary entry in release archive")?;
    let mut binary = File::open(binary_path)
        .with_context(|| format!("failed to open release binary {}", binary_path.display()))?;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = binary
            .read(&mut buffer)
            .with_context(|| format!("failed to read release binary {}", binary_path.display()))?;
        if read == 0 {
            break;
        }
        zip.write_all(&buffer[..read])
            .context("failed to write binary into release archive")?;
    }

    let manifest_options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);
    zip.start_file("release-manifest.json", manifest_options)
        .context("failed to start manifest entry in release archive")?;
    zip.write_all(manifest_text.as_bytes())
        .context("failed to write manifest into release archive")?;
    zip.finish().context("failed to finish release archive")?;
    Ok(())
}

pub(super) fn validate_archive_entries(
    archive: &mut ZipArchive<File>,
    expected_binary_name: Option<&str>,
) -> Result<()> {
    if archive.len() != EXPECTED_ARCHIVE_ENTRIES {
        anyhow::bail!(
            "release archive must contain exactly {EXPECTED_ARCHIVE_ENTRIES} entries, got {}",
            archive.len()
        );
    }
    let mut names = HashSet::with_capacity(EXPECTED_ARCHIVE_ENTRIES);
    let mut expanded_bytes = 0u64;
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .with_context(|| format!("failed to inspect release archive entry {index}"))?;
        let name = entry.name().to_string();
        safe_file_name(&name, "archive entry")?;
        if !names.insert(name.clone()) {
            anyhow::bail!("release archive contains duplicate entry {name}");
        }
        let max_entry_bytes = if name == "release-manifest.json" {
            MAX_ARCHIVE_MANIFEST_BYTES
        } else {
            MAX_RELEASE_BINARY_BYTES
        };
        if entry.size() > max_entry_bytes {
            anyhow::bail!("release archive entry {name} exceeds {max_entry_bytes} bytes");
        }
        expanded_bytes = expanded_bytes
            .checked_add(entry.size())
            .context("release archive expanded byte count overflow")?;
        if expanded_bytes > MAX_ARCHIVE_EXPANDED_BYTES {
            anyhow::bail!("release archive exceeds {MAX_ARCHIVE_EXPANDED_BYTES} expanded bytes");
        }
        let compressed = entry.compressed_size();
        if entry.size() > 0
            && (compressed == 0 || entry.size() > compressed.saturating_mul(MAX_COMPRESSION_RATIO))
        {
            anyhow::bail!(
                "release archive entry {name} exceeds {MAX_COMPRESSION_RATIO}:1 compression ratio"
            );
        }
    }
    if !names.contains("release-manifest.json") {
        anyhow::bail!("release archive is missing release-manifest.json");
    }
    if let Some(binary_name) = expected_binary_name {
        let expected =
            HashSet::from(["release-manifest.json".to_string(), binary_name.to_string()]);
        if names != expected {
            anyhow::bail!(
                "release archive entries do not exactly match the manifest binary and release-manifest.json"
            );
        }
    }
    Ok(())
}

pub(super) fn read_archive_manifest(
    archive: &mut ZipArchive<File>,
) -> Result<ReleaseArchiveManifestOwned> {
    let mut manifest_entry = archive
        .by_name("release-manifest.json")
        .context("release archive is missing release-manifest.json")?;
    let mut manifest_bytes = Vec::new();
    manifest_entry
        .by_ref()
        .take(MAX_ARCHIVE_MANIFEST_BYTES + 1)
        .read_to_end(&mut manifest_bytes)
        .context("failed to read release-manifest.json from archive")?;
    if manifest_bytes.len() as u64 > MAX_ARCHIVE_MANIFEST_BYTES {
        anyhow::bail!(
            "release-manifest.json exceeds {MAX_ARCHIVE_MANIFEST_BYTES} byte verification limit"
        );
    }
    serde_json::from_slice(&manifest_bytes).context("failed to parse release-manifest.json")
}

pub(super) fn hash_archive_entry(
    archive: &mut ZipArchive<File>,
    entry_name: &str,
) -> Result<(String, u64)> {
    let mut entry = archive
        .by_name(entry_name)
        .with_context(|| format!("release archive is missing binary entry {entry_name}"))?;
    if entry.size() > MAX_RELEASE_BINARY_BYTES {
        anyhow::bail!(
            "release binary entry exceeds {MAX_RELEASE_BINARY_BYTES} byte verification limit"
        );
    }
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    let mut len = 0u64;
    loop {
        let read = entry
            .read(&mut buffer)
            .with_context(|| format!("failed to read release archive entry {entry_name}"))?;
        if read == 0 {
            break;
        }
        len = len
            .checked_add(read as u64)
            .context("release binary expanded byte count overflow")?;
        if len > MAX_RELEASE_BINARY_BYTES {
            anyhow::bail!(
                "release binary entry exceeds {MAX_RELEASE_BINARY_BYTES} byte verification limit"
            );
        }
        hasher.update(&buffer[..read]);
    }
    let mut sha256 = String::with_capacity(64);
    for byte in hasher.finalize() {
        use std::fmt::Write as _;
        write!(&mut sha256, "{byte:02x}")
            .map_err(|_| anyhow::anyhow!("failed to format release archive digest"))?;
    }
    Ok((sha256, len))
}
