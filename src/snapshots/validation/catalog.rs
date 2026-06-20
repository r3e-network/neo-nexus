use anyhow::{Context, Result};
use ed25519_dalek::VerifyingKey;

use super::download::validate_snapshot_download_request;
use crate::snapshots::{
    catalog_io::{decode_fixed_base64, is_https_source, optional_trimmed, SnapshotCatalogSource},
    FastSyncSnapshotCatalogEntry, SnapshotCatalogLoadRequest,
};

pub fn validate_snapshot_catalog_load_request(request: &SnapshotCatalogLoadRequest) -> Result<()> {
    let source = request.source.trim();
    if source.is_empty() {
        anyhow::bail!("snapshot catalog source is required");
    }
    SnapshotCatalogSource::parse(source, "snapshot catalog source")?;
    if request.max_bytes == 0 {
        anyhow::bail!("snapshot catalog size limit must be greater than 0");
    }

    let signature_source = optional_trimmed(&request.signature_source);
    let public_key = optional_trimmed(&request.ed25519_public_key);
    if signature_source.is_some() != public_key.is_some() {
        anyhow::bail!(
            "snapshot catalog signature source and Ed25519 public key must be provided together"
        );
    }
    if let Some(signature_source) = signature_source {
        SnapshotCatalogSource::parse(signature_source, "snapshot catalog signature")?;
    }
    if let Some(public_key) = public_key {
        let key_bytes = decode_fixed_base64::<32>("Ed25519 public key", public_key)?;
        VerifyingKey::from_bytes(&key_bytes).context("invalid Ed25519 public key")?;
    }
    if is_https_source(source) && signature_source.is_none() {
        anyhow::bail!("remote snapshot catalogs require an Ed25519 signature and public key");
    }
    Ok(())
}

pub fn validate_snapshot_catalog_entry(entry: &FastSyncSnapshotCatalogEntry) -> Result<()> {
    if entry.id.trim().is_empty() {
        anyhow::bail!("snapshot catalog entry id is required");
    }
    if entry.label.trim().is_empty() {
        anyhow::bail!("snapshot catalog entry label is required");
    }
    validate_snapshot_download_request(&entry.download_request())?;
    Ok(())
}
