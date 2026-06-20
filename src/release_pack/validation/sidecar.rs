use anyhow::Result;

use super::{
    super::manifest::ReleaseSidecarManifestOwned,
    names::{safe_file_name, validate_sha256},
};

pub(in crate::release_pack) fn validate_sidecar_manifest(
    manifest: &ReleaseSidecarManifestOwned,
) -> Result<()> {
    if manifest.schema_version != 1 {
        anyhow::bail!(
            "unsupported release package manifest schema {}",
            manifest.schema_version
        );
    }
    if manifest.application != "NeoNexus" {
        anyhow::bail!(
            "release package application must be NeoNexus, got {}",
            manifest.application
        );
    }
    if manifest.package_id.trim().is_empty()
        || manifest.version.trim().is_empty()
        || manifest.os.trim().is_empty()
        || manifest.arch.trim().is_empty()
    {
        anyhow::bail!("release package manifest has incomplete identity fields");
    }
    safe_file_name(&manifest.archive_file, "archive file")?;
    safe_file_name(&manifest.binary_name, "binary name")?;
    validate_sha256(&manifest.archive_sha256, "archive SHA-256")?;
    validate_sha256(&manifest.binary_sha256, "binary SHA-256")?;
    if manifest.archive_bytes == 0 {
        anyhow::bail!("release archive byte count must be greater than zero");
    }
    if manifest.binary_bytes == 0 {
        anyhow::bail!("release binary byte count must be greater than zero");
    }
    Ok(())
}
