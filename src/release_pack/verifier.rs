use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use zip::ZipArchive;

use crate::snapshots::sha256_file;

use super::{
    archive::{hash_archive_entry, read_archive_manifest, validate_archive_entries},
    manifest::ReleaseSidecarManifestOwned,
    model::ReleasePackageVerification,
    validation::{
        resolve_release_manifest, safe_file_name, validate_archive_manifest,
        validate_checksum_file, validate_sidecar_manifest, MAX_RELEASE_ARCHIVE_BYTES,
    },
};

const MAX_SIDECAR_MANIFEST_BYTES: u64 = 64 * 1024;
const MAX_SIGNATURE_FILE_BYTES: u64 = 512;

pub struct ReleasePackageVerifier;

impl ReleasePackageVerifier {
    /// Verify hashes, sizes, names, and ZIP structure. This deliberately makes
    /// no publisher-authenticity claim.
    pub fn verify_integrity(input: impl AsRef<Path>) -> Result<ReleasePackageVerification> {
        Self::verify_inner(input.as_ref(), None)
    }

    /// Authenticate the canonical sidecar manifest with an explicit Ed25519
    /// trust anchor before performing all integrity checks.
    pub fn verify_authenticated(
        input: impl AsRef<Path>,
        trusted_public_key_base64: &str,
        detached_signature_path: impl AsRef<Path>,
    ) -> Result<ReleasePackageVerification> {
        if trusted_public_key_base64.trim().is_empty() {
            anyhow::bail!("trusted Ed25519 public key is required for authenticated verification");
        }
        Self::verify_inner(
            input.as_ref(),
            Some((trusted_public_key_base64, detached_signature_path.as_ref())),
        )
    }

    fn verify_inner(
        input: &Path,
        authentication: Option<(&str, &Path)>,
    ) -> Result<ReleasePackageVerification> {
        let manifest_path = resolve_release_manifest(input)?;
        let manifest_bytes = read_bounded_file(
            &manifest_path,
            MAX_SIDECAR_MANIFEST_BYTES,
            "release package manifest",
        )?;
        let sidecar: ReleaseSidecarManifestOwned = serde_json::from_slice(&manifest_bytes)
            .with_context(|| {
                format!(
                    "failed to parse release package manifest {}",
                    manifest_path.display()
                )
            })?;
        validate_sidecar_manifest(&sidecar)?;
        let canonical_manifest = serde_json::to_vec_pretty(&sidecar)
            .context("failed to canonicalize release package manifest")?;
        if manifest_bytes != canonical_manifest {
            anyhow::bail!(
                "release package manifest is not in the canonical NeoNexus JSON encoding"
            );
        }
        let publisher_authenticated = if let Some((trusted_key, signature_path)) = authentication {
            verify_manifest_signature(&canonical_manifest, trusted_key, signature_path)?;
            true
        } else {
            false
        };

        let manifest_dir = manifest_path
            .parent()
            .context("release package manifest must have a parent directory")?;
        let archive_file_name = safe_file_name(&sidecar.archive_file, "archive file")?;
        let archive_path = manifest_dir.join(archive_file_name);
        if !archive_path.is_file() {
            anyhow::bail!("release archive {} is missing", archive_path.display());
        }
        let archive_metadata = fs::metadata(&archive_path).with_context(|| {
            format!(
                "failed to inspect release archive {}",
                archive_path.display()
            )
        })?;
        if archive_metadata.len() > MAX_RELEASE_ARCHIVE_BYTES {
            anyhow::bail!(
                "release archive exceeds {MAX_RELEASE_ARCHIVE_BYTES} byte verification limit"
            );
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
        validate_archive_entries(&mut archive, None)?;

        let archive_manifest = read_archive_manifest(&mut archive)?;
        validate_archive_manifest(&archive_manifest, &sidecar)?;
        let binary_name = safe_file_name(&archive_manifest.binary_name, "binary name")?;
        validate_archive_entries(&mut archive, Some(binary_name))?;
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
            publisher_authenticated,
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

fn verify_manifest_signature(
    canonical_manifest: &[u8],
    trusted_public_key_base64: &str,
    detached_signature_path: &Path,
) -> Result<()> {
    if !detached_signature_path.is_file() {
        anyhow::bail!(
            "detached release signature {} is missing",
            detached_signature_path.display()
        );
    }
    let key_bytes = STANDARD
        .decode(trusted_public_key_base64.trim())
        .context("trusted Ed25519 public key must be base64")?;
    let key_array: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("trusted Ed25519 public key must decode to 32 bytes"))?;
    let verifying_key =
        VerifyingKey::from_bytes(&key_array).context("trusted Ed25519 public key is not valid")?;

    let signature_text = read_bounded_file(
        detached_signature_path,
        MAX_SIGNATURE_FILE_BYTES,
        "detached release signature",
    )?;
    let signature_bytes = STANDARD
        .decode(
            std::str::from_utf8(&signature_text)
                .context("detached release signature must be base64 UTF-8")?
                .trim(),
        )
        .context("detached release signature must be base64")?;
    let signature = Signature::from_slice(&signature_bytes)
        .context("detached Ed25519 signature must decode to 64 bytes")?;
    verifying_key
        .verify(canonical_manifest, &signature)
        .context("release publisher signature verification failed")
}

fn read_bounded_file(path: &Path, max_bytes: u64, label: &str) -> Result<Vec<u8>> {
    let mut file =
        File::open(path).with_context(|| format!("failed to open {label} {}", path.display()))?;
    let mut bytes = Vec::new();
    file.by_ref()
        .take(max_bytes + 1)
        .read_to_end(&mut bytes)
        .with_context(|| format!("failed to read {label} {}", path.display()))?;
    if bytes.len() as u64 > max_bytes {
        anyhow::bail!("{label} exceeds {max_bytes} byte verification limit");
    }
    Ok(bytes)
}
