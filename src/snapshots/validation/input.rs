use anyhow::Result;

use super::{
    download::validate_snapshot_download_request, hash::normalize_sha256, names::file_name_from_url,
};
use crate::snapshots::{NewFastSyncSnapshot, SnapshotDownloadRequest};

pub fn validate_snapshot_input(input: &NewFastSyncSnapshot) -> Result<()> {
    if input.id.trim().is_empty() {
        anyhow::bail!("snapshot id is required");
    }
    if input.label.trim().is_empty() {
        anyhow::bail!("snapshot label is required");
    }
    let has_source_path = !input.source_path.as_os_str().is_empty();
    let has_source_url = input
        .source_url
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty());
    if !has_source_path && !has_source_url {
        anyhow::bail!("snapshot source path or HTTPS URL is required");
    }
    if has_source_url {
        let request = SnapshotDownloadRequest {
            snapshot_id: input.id.clone(),
            url: input.source_url.clone().unwrap_or_default(),
            file_name: input
                .download_file_name
                .clone()
                .or_else(|| {
                    input
                        .source_url
                        .as_deref()
                        .and_then(|url| file_name_from_url(url).ok())
                })
                .unwrap_or_else(|| "snapshot.acc".to_string()),
            expected_sha256: input.expected_sha256.clone(),
            max_bytes: input.download_max_bytes,
        };
        validate_snapshot_download_request(&request)?;
    }
    normalize_sha256(&input.expected_sha256)?;
    Ok(())
}
