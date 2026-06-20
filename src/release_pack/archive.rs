use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use anyhow::{Context, Result};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

use crate::snapshots::sha256_bytes;

use super::{manifest::ReleaseArchiveManifestOwned, validation::safe_file_name};

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

pub(super) fn validate_archive_entry_names(archive: &mut ZipArchive<File>) -> Result<()> {
    let mut names = Vec::new();
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .with_context(|| format!("failed to inspect release archive entry {index}"))?;
        let name = entry.name().to_string();
        safe_file_name(&name, "archive entry")?;
        if names.iter().any(|existing| existing == &name) {
            anyhow::bail!("release archive contains duplicate entry {name}");
        }
        names.push(name);
    }
    if !names.iter().any(|name| name == "release-manifest.json") {
        anyhow::bail!("release archive is missing release-manifest.json");
    }
    Ok(())
}

pub(super) fn read_archive_manifest(
    archive: &mut ZipArchive<File>,
) -> Result<ReleaseArchiveManifestOwned> {
    let mut manifest_entry = archive
        .by_name("release-manifest.json")
        .context("release archive is missing release-manifest.json")?;
    let mut manifest_text = String::new();
    manifest_entry
        .read_to_string(&mut manifest_text)
        .context("failed to read release-manifest.json from archive")?;
    serde_json::from_str(&manifest_text).context("failed to parse release-manifest.json")
}

pub(super) fn hash_archive_entry(
    archive: &mut ZipArchive<File>,
    entry_name: &str,
) -> Result<(String, u64)> {
    let mut entry = archive
        .by_name(entry_name)
        .with_context(|| format!("release archive is missing binary entry {entry_name}"))?;
    let mut bytes = Vec::new();
    entry
        .read_to_end(&mut bytes)
        .with_context(|| format!("failed to read release archive entry {entry_name}"))?;
    let len = bytes.len() as u64;
    Ok((sha256_bytes(&bytes), len))
}
