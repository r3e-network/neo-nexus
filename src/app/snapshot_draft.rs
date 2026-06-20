use std::path::PathBuf;

use anyhow::Result;

use crate::{
    snapshots::{FastSyncSnapshotManager, NewFastSyncSnapshot},
    types::{Network, NodeType},
};

#[derive(Debug, Clone)]
pub(super) struct SnapshotDraft {
    pub(super) id: String,
    pub(super) label: String,
    pub(super) network: Network,
    pub(super) node_type: NodeType,
    pub(super) source_path: String,
    pub(super) source_url: String,
    pub(super) download_file_name: String,
    pub(super) download_max_mib: u64,
    pub(super) expected_sha256: String,
}

impl SnapshotDraft {
    pub(super) fn to_new_snapshot(&self) -> Result<NewFastSyncSnapshot> {
        Ok(NewFastSyncSnapshot {
            id: self.id.trim().to_string(),
            label: self.label.trim().to_string(),
            network: self.network,
            node_type: self.node_type,
            source_path: PathBuf::from(self.source_path.trim()),
            source_url: optional_string(&self.source_url),
            download_file_name: optional_string(&self.download_file_name),
            download_max_bytes: self.download_max_mib.saturating_mul(1024 * 1024),
            expected_sha256: self.expected_sha256.trim().to_string(),
        })
    }
}

impl Default for SnapshotDraft {
    fn default() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            network: Network::Testnet,
            node_type: NodeType::NeoRs,
            source_path: String::new(),
            source_url: String::new(),
            download_file_name: String::new(),
            download_max_mib: FastSyncSnapshotManager::DEFAULT_DOWNLOAD_MAX_BYTES / (1024 * 1024),
            expected_sha256: String::new(),
        }
    }
}

fn optional_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
