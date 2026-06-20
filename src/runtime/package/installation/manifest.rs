use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::runtime::RuntimeInstallation;

pub(super) fn write_install_manifest(
    install_dir: &Path,
    installation: &RuntimeInstallation,
) -> Result<PathBuf> {
    let path = install_dir.join("runtime-install.json");
    let manifest = RuntimeInstallManifest {
        package_id: &installation.package_id,
        label: &installation.label,
        node_type: installation.node_type.to_string(),
        version: &installation.version,
        platform: installation.platform.to_string(),
        binary_path: installation.binary_path.display().to_string(),
        sha256: &installation.sha256,
        signature_verified: installation.signature_verified,
        signer_public_key: installation.signer_public_key.as_deref(),
        bytes: installation.bytes,
        installed_at_unix: installation.installed_at_unix,
    };
    let text =
        serde_json::to_string_pretty(&manifest).context("failed to render runtime manifest")?;
    fs::write(&path, text.as_bytes())
        .with_context(|| format!("failed to write runtime manifest {}", path.display()))?;
    Ok(path)
}

#[derive(Serialize)]
struct RuntimeInstallManifest<'a> {
    package_id: &'a str,
    label: &'a str,
    node_type: String,
    version: &'a str,
    platform: String,
    binary_path: String,
    sha256: &'a str,
    signature_verified: bool,
    signer_public_key: Option<&'a str>,
    bytes: u64,
    installed_at_unix: u64,
}
