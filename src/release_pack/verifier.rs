use std::{
    fs::{self, File},
    path::Path,
};

use anyhow::{Context, Result};
use zip::ZipArchive;

use crate::snapshots::sha256_file;

use super::{
    archive::{hash_archive_entry, read_archive_manifest, validate_archive_entry_names},
    manifest::ReleaseSidecarManifestOwned,
    model::ReleasePackageVerification,
    validation::{
        resolve_release_manifest, safe_file_name, validate_archive_manifest,
        validate_checksum_file, validate_sidecar_manifest,
    },
};

pub struct ReleasePackageVerifier;

impl ReleasePackageVerifier {
    pub fn verify(input: impl AsRef<Path>) -> Result<ReleasePackageVerification> {
        let manifest_path = resolve_release_manifest(input.as_ref())?;
        let manifest_dir = manifest_path
            .parent()
            .context("release package manifest must have a parent directory")?;
        let sidecar: ReleaseSidecarManifestOwned =
            serde_json::from_str(&fs::read_to_string(&manifest_path).with_context(|| {
                format!(
                    "failed to read release package manifest {}",
                    manifest_path.display()
                )
            })?)
            .with_context(|| {
                format!(
                    "failed to parse release package manifest {}",
                    manifest_path.display()
                )
            })?;
        validate_sidecar_manifest(&sidecar)?;

        let archive_file_name = safe_file_name(&sidecar.archive_file, "archive file")?;
        let archive_path = manifest_dir.join(archive_file_name);
        if !archive_path.is_file() {
            anyhow::bail!("release archive {} is missing", archive_path.display());
        }
        let checksum_path = manifest_dir.join(format!("{archive_file_name}.sha256"));
        if !checksum_path.is_file() {
            anyhow::bail!("release checksum {} is missing", checksum_path.display());
        }

        let (archive_sha256, archive_bytes) = sha256_file(&archive_path)?;
        if archive_sha256 != sidecar.archive_sha256 {
            anyhow::bail!(
                "release archive SHA-256 mismatch: expected {}, got {}",
                sidecar.archive_sha256,
                archive_sha256
            );
        }
        if archive_bytes != sidecar.archive_bytes {
            anyhow::bail!(
                "release archive byte count mismatch: expected {}, got {}",
                sidecar.archive_bytes,
                archive_bytes
            );
        }
        validate_checksum_file(&checksum_path, archive_file_name, &archive_sha256)?;

        let archive_file = File::open(&archive_path).with_context(|| {
            format!("failed to open release archive {}", archive_path.display())
        })?;
        let mut archive = ZipArchive::new(archive_file).with_context(|| {
            format!("failed to read release archive {}", archive_path.display())
        })?;
        validate_archive_entry_names(&mut archive)?;

        let archive_manifest = read_archive_manifest(&mut archive)?;
        validate_archive_manifest(&archive_manifest, &sidecar)?;
        let binary_name = safe_file_name(&archive_manifest.binary_name, "binary name")?;
        let (binary_sha256, binary_bytes) = hash_archive_entry(&mut archive, binary_name)?;
        if binary_sha256 != sidecar.binary_sha256 {
            anyhow::bail!(
                "release binary SHA-256 mismatch: expected {}, got {}",
                sidecar.binary_sha256,
                binary_sha256
            );
        }
        if binary_bytes != sidecar.binary_bytes {
            anyhow::bail!(
                "release binary byte count mismatch: expected {}, got {}",
                sidecar.binary_bytes,
                binary_bytes
            );
        }

        Ok(ReleasePackageVerification {
            archive_path,
            checksum_path,
            manifest_path,
            package_id: sidecar.package_id,
            archive_sha256,
            archive_bytes,
            binary_name: binary_name.to_string(),
            binary_sha256,
            binary_bytes,
        })
    }
}
