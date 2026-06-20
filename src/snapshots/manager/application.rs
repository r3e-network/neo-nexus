use std::path::Path;

use anyhow::Result;

use crate::types::NodeConfig;

use super::FastSyncSnapshotManager;
use crate::snapshots::{
    import::{self, SnapshotApplication},
    model::FastSyncSnapshot,
};

impl FastSyncSnapshotManager {
    pub fn apply_to_node(
        snapshot: &FastSyncSnapshot,
        node: &NodeConfig,
        node_data_dir: impl AsRef<Path>,
    ) -> Result<SnapshotApplication> {
        import::apply_to_node(snapshot, node, node_data_dir.as_ref())
    }
}
