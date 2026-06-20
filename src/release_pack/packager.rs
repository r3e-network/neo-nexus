use std::{env, path::Path};

use anyhow::{Context, Result};

use super::{model::ReleasePackage, platform::ReleasePackagePlatform};

mod draft;
mod files;

use self::{
    draft::PackageDraft,
    files::{
        archive_digest, ensure_output_dir, file_name_or, publish_archive, resolve_binary_path,
        write_checksum, write_sidecar_manifest,
    },
};

pub struct ReleasePackager;

impl ReleasePackager {
    pub fn package_current_executable(output_dir: impl AsRef<Path>) -> Result<ReleasePackage> {
        let executable =
            env::current_exe().context("failed to locate the running NeoNexus executable")?;
        Self::package_binary(
            executable,
            output_dir,
            env!("CARGO_PKG_VERSION"),
            ReleasePackagePlatform::current(),
        )
    }

    pub fn package_binary(
        binary_path: impl AsRef<Path>,
        output_dir: impl AsRef<Path>,
        version: &str,
        platform: ReleasePackagePlatform,
    ) -> Result<ReleasePackage> {
        let output_dir = output_dir.as_ref();
        let binary_path = resolve_binary_path(binary_path.as_ref())?;
        ensure_output_dir(output_dir)?;
        let draft = PackageDraft::new(binary_path, version, platform)?;

        let archive_manifest_text = draft.archive_manifest_text()?;
        let archive_path = publish_archive(output_dir, &draft, &archive_manifest_text)?;
        let (archive_sha256, archive_bytes) = archive_digest(&archive_path)?;
        let archive_file_name = file_name_or(&archive_path, "neo-nexus.zip");
        let manifest_path = write_sidecar_manifest(
            output_dir,
            &draft,
            &archive_file_name,
            &archive_sha256,
            archive_bytes,
        )?;
        let checksum_path =
            write_checksum(output_dir, &draft, &archive_file_name, &archive_sha256)?;

        Ok(ReleasePackage {
            archive_path,
            checksum_path,
            manifest_path,
            archive_sha256,
            archive_bytes,
            binary_sha256: draft.binary_sha256().to_string(),
            binary_bytes: draft.binary_bytes(),
            package_id: draft.package_id().to_string(),
        })
    }
}
