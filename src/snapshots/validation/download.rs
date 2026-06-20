use anyhow::{Context, Result};
use url::Url;

use super::{hash::normalize_sha256, names::safe_file_name};
use crate::snapshots::SnapshotDownloadRequest;

pub fn validate_snapshot_download_request(request: &SnapshotDownloadRequest) -> Result<Url> {
    if request.snapshot_id.trim().is_empty() {
        anyhow::bail!("snapshot id is required");
    }
    let url = Url::parse(request.url.trim()).context("snapshot download URL is invalid")?;
    if url.scheme() != "https" {
        anyhow::bail!("snapshot download URL must use HTTPS");
    }
    if url.host_str().is_none() {
        anyhow::bail!("snapshot download URL must include a host");
    }
    safe_file_name(&request.file_name)?;
    normalize_sha256(&request.expected_sha256)?;
    if request.max_bytes == 0 {
        anyhow::bail!("snapshot download size limit must be greater than 0");
    }
    Ok(url)
}

pub fn validate_snapshot_https_redirect(current: &Url, location: &str) -> Result<Url> {
    let next = current
        .join(location)
        .with_context(|| format!("invalid snapshot download redirect from {current}"))?;
    if next.scheme() != "https" {
        anyhow::bail!("snapshot download redirect must stay on HTTPS");
    }
    Ok(next)
}
