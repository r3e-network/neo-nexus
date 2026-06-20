use anyhow::Result;

use super::super::manifest::{ReleaseArchiveManifestOwned, ReleaseSidecarManifestOwned};

pub(in crate::release_pack) fn validate_archive_manifest(
    archive_manifest: &ReleaseArchiveManifestOwned,
    sidecar: &ReleaseSidecarManifestOwned,
) -> Result<()> {
    if archive_manifest.schema_version != 1 {
        anyhow::bail!(
            "unsupported archive manifest schema {}",
            archive_manifest.schema_version
        );
    }
    if archive_manifest.application != "NeoNexus" {
        anyhow::bail!(
            "archive manifest application must be NeoNexus, got {}",
            archive_manifest.application
        );
    }
    if archive_manifest.package_id != sidecar.package_id
        || archive_manifest.version != sidecar.version
        || archive_manifest.os != sidecar.os
        || archive_manifest.arch != sidecar.arch
        || archive_manifest.binary_name != sidecar.binary_name
        || archive_manifest.binary_sha256 != sidecar.binary_sha256
        || archive_manifest.binary_bytes != sidecar.binary_bytes
    {
        anyhow::bail!("archive manifest does not match release sidecar manifest");
    }
    Ok(())
}
