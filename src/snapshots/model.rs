use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::types::{Network, NodeType};

use super::validation::file_name_from_url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewFastSyncSnapshot {
    pub id: String,
    pub label: String,
    pub network: Network,
    pub node_type: NodeType,
    pub source_path: PathBuf,
    pub source_url: Option<String>,
    pub download_file_name: Option<String>,
    pub download_max_bytes: u64,
    pub expected_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FastSyncSnapshot {
    pub id: String,
    pub label: String,
    pub network: Network,
    pub node_type: NodeType,
    pub source_path: PathBuf,
    pub source_url: Option<String>,
    pub download_file_name: Option<String>,
    pub download_max_bytes: u64,
    pub expected_sha256: String,
    pub cached_path: Option<PathBuf>,
    pub verified_sha256: Option<String>,
    pub verified_at_unix: Option<u64>,
    pub bytes: Option<u64>,
}

impl FastSyncSnapshot {
    pub fn download_request(&self) -> Result<SnapshotDownloadRequest> {
        let url = self
            .source_url
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .context("snapshot HTTPS URL is required")?;
        Ok(SnapshotDownloadRequest {
            snapshot_id: self.id.clone(),
            url: url.to_string(),
            file_name: self.download_file_name.clone().unwrap_or_else(|| {
                file_name_from_url(url).unwrap_or_else(|_| "snapshot.acc".to_string())
            }),
            expected_sha256: self.expected_sha256.clone(),
            max_bytes: self.download_max_bytes,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotDownloadRequest {
    pub snapshot_id: String,
    pub url: String,
    pub file_name: String,
    pub expected_sha256: String,
    pub max_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotCatalogLoadRequest {
    pub source: String,
    pub signature_source: Option<String>,
    pub ed25519_public_key: Option<String>,
    pub max_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotCatalogLoad {
    pub catalog: super::FastSyncSnapshotCatalog,
    pub source: String,
    pub bytes: u64,
    pub signature_verified: Option<bool>,
    pub loaded_at_unix: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotVerification {
    pub sha256: String,
    pub expected_sha256: String,
    pub bytes: u64,
    pub matches: bool,
    pub verified_at_unix: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotCache {
    pub path: PathBuf,
    pub sha256: String,
    pub bytes: u64,
    pub cached_at_unix: u64,
}
