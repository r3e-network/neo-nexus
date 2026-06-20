use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::snapshots::sha256_file;

use super::super::archive::write_release_archive;
use super::draft::PackageDraft;

pub(super) fn resolve_binary_path(binary_path: &Path) -> Result<PathBuf> {
    let binary_path = binary_path
        .canonicalize()
        .with_context(|| format!("failed to resolve release binary {}", binary_path.display()))?;
    if !binary_path.is_file() {
        anyhow::bail!("release binary {} is not a file", binary_path.display());
    }
    Ok(binary_path)
}

pub(super) fn ensure_output_dir(output_dir: &Path) -> Result<()> {
    fs::create_dir_all(output_dir).with_context(|| {
        format!(
            "failed to create release package directory {}",
            output_dir.display()
        )
    })
}

pub(super) fn publish_archive(
    output_dir: &Path,
    draft: &PackageDraft,
    archive_manifest_text: &str,
) -> Result<PathBuf> {
    let archive_path = output_dir.join(draft.archive_file_name());
    let temp_archive_path = output_dir.join(draft.temporary_archive_file_name());
    write_release_archive(
        &temp_archive_path,
        draft.binary_path(),
        draft.binary_name(),
        archive_manifest_text,
    )?;
    if archive_path.exists() {
        fs::remove_file(&archive_path).with_context(|| {
            format!(
                "failed to replace existing release archive {}",
                archive_path.display()
            )
        })?;
    }
    fs::rename(&temp_archive_path, &archive_path).with_context(|| {
        format!(
            "failed to publish release archive {}",
            archive_path.display()
        )
    })?;
    Ok(archive_path)
}

pub(super) fn archive_digest(archive_path: &Path) -> Result<(String, u64)> {
    sha256_file(archive_path)
}

pub(super) fn write_sidecar_manifest(
    output_dir: &Path,
    draft: &PackageDraft,
    archive_file_name: &str,
    archive_sha256: &str,
    archive_bytes: u64,
) -> Result<PathBuf> {
    let manifest_path = output_dir.join(format!("{}.manifest.json", draft.package_id()));
    let sidecar_manifest_text =
        draft.sidecar_manifest_text(archive_file_name, archive_sha256, archive_bytes)?;
    fs::write(&manifest_path, sidecar_manifest_text.as_bytes()).with_context(|| {
        format!(
            "failed to write release package manifest {}",
            manifest_path.display()
        )
    })?;
    Ok(manifest_path)
}

pub(super) fn write_checksum(
    output_dir: &Path,
    draft: &PackageDraft,
    archive_file_name: &str,
    archive_sha256: &str,
) -> Result<PathBuf> {
    let checksum_path = output_dir.join(format!("{}.zip.sha256", draft.package_id()));
    fs::write(
        &checksum_path,
        format!("{archive_sha256}  {archive_file_name}\n").as_bytes(),
    )
    .with_context(|| {
        format!(
            "failed to write release package checksum {}",
            checksum_path.display()
        )
    })?;
    Ok(checksum_path)
}

pub(super) fn file_name_or(path: &Path, fallback: &str) -> String {
    path.file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or(fallback)
        .to_string()
}
