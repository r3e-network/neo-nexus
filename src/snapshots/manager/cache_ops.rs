use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use super::FastSyncSnapshotManager;
use crate::snapshots::{
    cache::{cache_filename, cache_filename_parts},
    catalog_io::fetch_https_response,
    current_unix_time,
    io::{copy_file_hashed, copy_reader_hashed, replace_file},
    model::{FastSyncSnapshot, SnapshotCache, SnapshotDownloadRequest},
    validation::{
        normalize_sha256, safe_file_name, validate_snapshot_download_request, verified_source_path,
    },
};

impl FastSyncSnapshotManager {
    pub fn download_https(
        request: &SnapshotDownloadRequest,
        cache_dir: impl AsRef<Path>,
    ) -> Result<SnapshotCache> {
        let url = validate_snapshot_download_request(request)?;
        let response = fetch_https_response(url)?;
        if let Some(content_length) = response.header("content-length") {
            let length = content_length
                .parse::<u64>()
                .context("invalid content-length header from snapshot download")?;
            if length > request.max_bytes {
                anyhow::bail!(
                    "snapshot download is too large: {length} bytes exceeds limit {}",
                    request.max_bytes
                );
            }
        }

        Self::cache_download_from_reader(request, cache_dir, response.into_reader())
    }

    pub fn cache_download_from_reader(
        request: &SnapshotDownloadRequest,
        cache_dir: impl AsRef<Path>,
        reader: impl Read,
    ) -> Result<SnapshotCache> {
        validate_snapshot_download_request(request)?;
        let expected_sha256 = normalize_sha256(&request.expected_sha256)?;
        let file_name = safe_file_name(&request.file_name)?;
        let cache_dir = cache_dir.as_ref();
        fs::create_dir_all(cache_dir)
            .with_context(|| format!("failed to create snapshot cache {}", cache_dir.display()))?;

        let target = cache_dir.join(cache_filename_parts(
            &request.snapshot_id,
            &file_name,
            &expected_sha256,
        ));
        publish_hashed_reader(reader, &target, &expected_sha256, request.max_bytes)
    }

    pub fn cache(
        snapshot: &FastSyncSnapshot,
        cache_dir: impl AsRef<Path>,
    ) -> Result<SnapshotCache> {
        let source = verified_source_path(&snapshot.source_path)?;
        let expected_sha256 = normalize_sha256(&snapshot.expected_sha256)?;
        let cache_dir = cache_dir.as_ref();
        fs::create_dir_all(cache_dir)
            .with_context(|| format!("failed to create snapshot cache {}", cache_dir.display()))?;

        let target = cache_dir.join(cache_filename(snapshot, &expected_sha256));
        publish_hashed_file(&source, &target, &expected_sha256)
    }
}

fn publish_hashed_reader(
    reader: impl Read,
    target: &Path,
    expected_sha256: &str,
    max_bytes: u64,
) -> Result<SnapshotCache> {
    let temp = download_temp_path(target);
    let (sha256, bytes) = copy_reader_hashed(reader, &temp, max_bytes)?;
    publish_verified_cache(temp, target.to_path_buf(), sha256, bytes, expected_sha256)
}

fn publish_hashed_file(
    source: &Path,
    target: &Path,
    expected_sha256: &str,
) -> Result<SnapshotCache> {
    let temp = download_temp_path(target);
    let (sha256, bytes) = copy_file_hashed(source, &temp)?;
    publish_verified_cache(temp, target.to_path_buf(), sha256, bytes, expected_sha256)
}

fn publish_verified_cache(
    temp: PathBuf,
    target: PathBuf,
    sha256: String,
    bytes: u64,
    expected_sha256: &str,
) -> Result<SnapshotCache> {
    if sha256 != expected_sha256 {
        let _ = fs::remove_file(&temp);
        anyhow::bail!("snapshot checksum mismatch: expected {expected_sha256}, got {sha256}");
    }

    replace_file(&temp, &target)?;

    Ok(SnapshotCache {
        path: target,
        sha256,
        bytes,
        cached_at_unix: current_unix_time()?,
    })
}

fn download_temp_path(target: &Path) -> PathBuf {
    target.with_extension("download")
}
