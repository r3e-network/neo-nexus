use std::{fs, io::Read, path::Path};

use anyhow::{Context, Result};

use crate::snapshots::normalize_sha256;

use super::super::{
    catalog::{self, RuntimeCatalogLoad, RuntimeCatalogLoadRequest},
    current_unix_time, fetch_https_response,
    io::{cache_file_name, copy_reader_hashed, replace_file, safe_file_name},
    validate_download_request, RuntimeDownload, RuntimeDownloadRequest,
};
use super::RuntimePackageManager;

impl RuntimePackageManager {
    pub fn download_https(
        request: &RuntimeDownloadRequest,
        download_dir: impl AsRef<Path>,
    ) -> Result<RuntimeDownload> {
        let url = validate_download_request(request)?;
        let response = fetch_https_response(url)?;
        if let Some(content_length) = response.header("content-length") {
            let length = content_length
                .parse::<u64>()
                .context("invalid content-length header from runtime download")?;
            if length > request.max_bytes {
                anyhow::bail!(
                    "runtime download is too large: {length} bytes exceeds limit {}",
                    request.max_bytes
                );
            }
        }

        Self::cache_download_from_reader(request, download_dir, response.into_reader())
    }

    pub fn load_release_catalog(request: &RuntimeCatalogLoadRequest) -> Result<RuntimeCatalogLoad> {
        catalog::load_release_catalog(request)
    }

    pub fn cache_download_from_reader(
        request: &RuntimeDownloadRequest,
        download_dir: impl AsRef<Path>,
        reader: impl Read,
    ) -> Result<RuntimeDownload> {
        validate_download_request(request)?;
        let expected_sha256 = normalize_sha256(&request.expected_sha256)?;
        let download_dir = download_dir.as_ref();
        fs::create_dir_all(download_dir).with_context(|| {
            format!(
                "failed to create runtime download directory {}",
                download_dir.display()
            )
        })?;

        let file_name = safe_file_name(&request.file_name)?;
        let target = download_dir.join(cache_file_name(&file_name, &expected_sha256));
        let temp = target.with_extension("download");
        let (sha256, bytes) = copy_reader_hashed(reader, &temp, request.max_bytes)?;
        if sha256 != expected_sha256 {
            let _ = fs::remove_file(&temp);
            anyhow::bail!(
                "runtime download checksum mismatch: expected {expected_sha256}, got {sha256}"
            );
        }
        replace_file(&temp, &target)?;

        Ok(RuntimeDownload {
            url: request.url.trim().to_string(),
            path: target,
            sha256,
            bytes,
            downloaded_at_unix: current_unix_time()?,
        })
    }
}
