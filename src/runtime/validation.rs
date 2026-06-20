use anyhow::{Context, Result};
use ed25519_dalek::VerifyingKey;
use url::Url;

use crate::snapshots::normalize_sha256;

use super::{
    decode_fixed_base64, io::safe_file_name, RuntimeDownloadRequest, RuntimePackageManifest,
    RuntimeSignerProfile,
};

pub fn validate_runtime_manifest(manifest: &RuntimePackageManifest) -> Result<()> {
    if manifest.id.trim().is_empty() {
        anyhow::bail!("runtime package id is required");
    }
    if manifest.label.trim().is_empty() {
        anyhow::bail!("runtime package label is required");
    }
    if manifest.version.trim().is_empty() {
        anyhow::bail!("runtime package version is required");
    }
    if manifest.platform.os.trim().is_empty() {
        anyhow::bail!("runtime package OS is required");
    }
    if manifest.platform.arch.trim().is_empty() {
        anyhow::bail!("runtime package architecture is required");
    }
    if manifest.source_path.as_os_str().is_empty() {
        anyhow::bail!("runtime package source path is required");
    }
    if manifest.signature_path.is_some() != manifest.ed25519_public_key.is_some() {
        anyhow::bail!("runtime signature path and Ed25519 public key must be provided together");
    }
    if let Some(signature_path) = &manifest.signature_path {
        if signature_path.as_os_str().is_empty() {
            anyhow::bail!("runtime signature path is required");
        }
    }
    if let Some(public_key) = &manifest.ed25519_public_key {
        decode_fixed_base64::<32>("Ed25519 public key", public_key)?;
    }
    safe_file_name(&manifest.executable_name)?;
    normalize_sha256(&manifest.expected_sha256)?;
    Ok(())
}

pub fn validate_download_request(request: &RuntimeDownloadRequest) -> Result<Url> {
    let url = Url::parse(request.url.trim()).context("runtime download URL is invalid")?;
    if url.scheme() != "https" {
        anyhow::bail!("runtime download URL must use HTTPS");
    }
    if url.host_str().is_none() {
        anyhow::bail!("runtime download URL must include a host");
    }
    safe_file_name(&request.file_name)?;
    normalize_sha256(&request.expected_sha256)?;
    if request.max_bytes == 0 {
        anyhow::bail!("runtime download size limit must be greater than 0");
    }
    Ok(url)
}

pub fn validate_runtime_signer_profile(profile: &RuntimeSignerProfile) -> Result<()> {
    if profile.id.trim().is_empty() {
        anyhow::bail!("runtime signer profile id is required");
    }
    if profile.label.trim().is_empty() {
        anyhow::bail!("runtime signer profile label is required");
    }
    if profile.created_at_unix == 0 {
        anyhow::bail!("runtime signer profile creation time is required");
    }
    let key_bytes = decode_fixed_base64::<32>("Ed25519 public key", &profile.ed25519_public_key)?;
    VerifyingKey::from_bytes(&key_bytes).context("invalid Ed25519 public key")?;
    Ok(())
}

pub fn validate_https_redirect(current: &Url, location: &str) -> Result<Url> {
    let next = current
        .join(location)
        .with_context(|| format!("invalid runtime download redirect from {current}"))?;
    if next.scheme() != "https" {
        anyhow::bail!("runtime download redirect must stay on HTTPS");
    }
    Ok(next)
}
