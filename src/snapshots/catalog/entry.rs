use std::path::PathBuf;

use crate::types::{Network, NodeType};

use super::parse::parse_catalog_json;
use crate::snapshots::model::{NewFastSyncSnapshot, SnapshotDownloadRequest};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FastSyncSnapshotCatalog {
    pub schema_version: u32,
    pub generated_at_unix: Option<u64>,
    pub snapshots: Vec<FastSyncSnapshotCatalogEntry>,
}

impl FastSyncSnapshotCatalog {
    pub fn from_json(text: &str) -> anyhow::Result<Self> {
        parse_catalog_json(text)
    }

    pub fn get(&self, id: &str) -> Option<&FastSyncSnapshotCatalogEntry> {
        self.snapshots.iter().find(|snapshot| snapshot.id == id)
    }

    pub fn compatible_entries(
        &self,
        network: Network,
        node_type: NodeType,
    ) -> Vec<&FastSyncSnapshotCatalogEntry> {
        self.snapshots
            .iter()
            .filter(|snapshot| snapshot.network == network && snapshot.node_type == node_type)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FastSyncSnapshotCatalogEntry {
    pub id: String,
    pub label: String,
    pub network: Network,
    pub node_type: NodeType,
    pub url: String,
    pub file_name: String,
    pub expected_sha256: String,
    pub max_bytes: u64,
}

impl FastSyncSnapshotCatalogEntry {
    pub fn download_request(&self) -> SnapshotDownloadRequest {
        SnapshotDownloadRequest {
            snapshot_id: self.id.clone(),
            url: self.url.clone(),
            file_name: self.file_name.clone(),
            expected_sha256: self.expected_sha256.clone(),
            max_bytes: self.max_bytes,
        }
    }

    pub fn to_new_snapshot(&self) -> NewFastSyncSnapshot {
        NewFastSyncSnapshot {
            id: self.id.clone(),
            label: self.label.clone(),
            network: self.network,
            node_type: self.node_type,
            source_path: PathBuf::new(),
            source_url: Some(self.url.clone()),
            download_file_name: Some(self.file_name.clone()),
            download_max_bytes: self.max_bytes,
            expected_sha256: self.expected_sha256.clone(),
        }
    }
}
