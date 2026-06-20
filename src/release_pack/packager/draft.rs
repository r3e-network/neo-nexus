use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::snapshots::sha256_file;

use super::super::{
    manifest::{ReleaseArchiveManifest, ReleaseSidecarManifest},
    platform::{release_binary_name, ReleasePackagePlatform},
    validation::safe_fragment,
};

pub(super) struct PackageDraft {
    binary_path: PathBuf,
    package_id: String,
    version: String,
    platform: ReleasePackagePlatform,
    binary_name: String,
    binary_sha256: String,
    binary_bytes: u64,
}

impl PackageDraft {
    pub(super) fn new(
        binary_path: PathBuf,
        version: &str,
        platform: ReleasePackagePlatform,
    ) -> Result<Self> {
        let version = safe_fragment(version);
        let package_id = format!("neo-nexus-{version}-{}", platform.id());
        let binary_name = release_binary_name(&platform);
        let (binary_sha256, binary_bytes) = sha256_file(&binary_path)?;
        Ok(Self {
            binary_path,
            package_id,
            version,
            platform,
            binary_name,
            binary_sha256,
            binary_bytes,
        })
    }

    pub(super) fn binary_path(&self) -> &Path {
        &self.binary_path
    }

    pub(super) fn package_id(&self) -> &str {
        &self.package_id
    }

    pub(super) fn binary_name(&self) -> &str {
        &self.binary_name
    }

    pub(super) fn binary_sha256(&self) -> &str {
        &self.binary_sha256
    }

    pub(super) fn binary_bytes(&self) -> u64 {
        self.binary_bytes
    }

    pub(super) fn archive_file_name(&self) -> String {
        format!("{}.zip", self.package_id)
    }

    pub(super) fn temporary_archive_file_name(&self) -> String {
        format!("{}.zip.tmp", self.package_id)
    }

    pub(super) fn archive_manifest_text(&self) -> Result<String> {
        let archive_manifest = ReleaseArchiveManifest {
            schema_version: 1,
            package_id: &self.package_id,
            application: "NeoNexus",
            version: &self.version,
            os: &self.platform.os,
            arch: &self.platform.arch,
            binary_name: &self.binary_name,
            binary_sha256: &self.binary_sha256,
            binary_bytes: self.binary_bytes,
        };
        serde_json::to_string_pretty(&archive_manifest)
            .context("failed to render release archive manifest")
    }

    pub(super) fn sidecar_manifest_text(
        &self,
        archive_file_name: &str,
        archive_sha256: &str,
        archive_bytes: u64,
    ) -> Result<String> {
        let sidecar_manifest = ReleaseSidecarManifest {
            schema_version: 1,
            package_id: &self.package_id,
            application: "NeoNexus",
            version: &self.version,
            os: &self.platform.os,
            arch: &self.platform.arch,
            archive_file: archive_file_name,
            archive_sha256,
            archive_bytes,
            binary_name: &self.binary_name,
            binary_sha256: &self.binary_sha256,
            binary_bytes: self.binary_bytes,
        };
        serde_json::to_string_pretty(&sidecar_manifest)
            .context("failed to render release package manifest")
    }
}
