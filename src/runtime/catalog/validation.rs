use std::path::PathBuf;

use anyhow::Result;

use super::super::{decode_fixed_base64, validate_download_request, validate_runtime_manifest};
use super::{
    io::{is_https_source, RuntimeCatalogSource},
    RuntimeCatalogLoadRequest, RuntimeCatalogProfile, RuntimeRelease,
};

pub fn validate_catalog_load_request(request: &RuntimeCatalogLoadRequest) -> Result<()> {
    let source = request.source.trim();
    if source.is_empty() {
        anyhow::bail!("runtime catalog source is required");
    }
    RuntimeCatalogSource::parse(source, "runtime catalog source")?;
    if request.max_bytes == 0 {
        anyhow::bail!("runtime catalog size limit must be greater than 0");
    }

    let signature_source = optional_trimmed(&request.signature_source);
    let public_key = optional_trimmed(&request.ed25519_public_key);
    if signature_source.is_some() != public_key.is_some() {
        anyhow::bail!(
            "runtime catalog signature source and Ed25519 public key must be provided together"
        );
    }
    if let Some(signature_source) = signature_source {
        RuntimeCatalogSource::parse(signature_source, "runtime catalog signature")?;
    }
    if let Some(public_key) = public_key {
        decode_fixed_base64::<32>("Ed25519 public key", public_key)?;
    }
    if is_https_source(source) && signature_source.is_none() {
        anyhow::bail!("remote runtime catalogs require an Ed25519 signature and public key");
    }

    Ok(())
}

pub fn validate_runtime_catalog_profile(profile: &RuntimeCatalogProfile) -> Result<()> {
    if profile.id.trim().is_empty() {
        anyhow::bail!("runtime catalog profile id is required");
    }
    if profile.label.trim().is_empty() {
        anyhow::bail!("runtime catalog profile label is required");
    }
    validate_catalog_load_request(&profile.load_request())?;
    Ok(())
}

pub fn validate_runtime_release(release: &RuntimeRelease) -> Result<()> {
    if release.id.trim().is_empty() {
        anyhow::bail!("runtime release id is required");
    }
    if release.label.trim().is_empty() {
        anyhow::bail!("runtime release label is required");
    }
    if release.version.trim().is_empty() {
        anyhow::bail!("runtime release version is required");
    }
    if release.platform.os.trim().is_empty() {
        anyhow::bail!("runtime release OS is required");
    }
    if release.platform.arch.trim().is_empty() {
        anyhow::bail!("runtime release architecture is required");
    }
    validate_download_request(&release.download_request())?;
    validate_runtime_manifest(&release.manifest_for_source(PathBuf::from("runtime-release")))?;
    Ok(())
}

pub(super) fn optional_trimmed(value: &Option<String>) -> Option<&str> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}
