use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
};

use crate::{
    supervisor::model::SyncProgress,
    types::{NodeConfig, NodeType},
};

/// In-memory, same-source observations. This store owns no files or workers.
#[derive(Clone, Default)]
pub struct LogObservations(Arc<Store>);

#[derive(Default)]
struct Store {
    collecting: AtomicBool,
    entries: RwLock<HashMap<String, Entry>>,
}

#[derive(Default)]
struct Entry {
    epoch: u64,
    source: Option<(NodeType, Option<u32>)>,
    sync: Option<Arc<SyncProgress>>,
}

impl LogObservations {
    pub(crate) fn sync_progress(&self, node: &NodeConfig) -> Option<Arc<SyncProgress>> {
        let entries = self.0.entries.read().unwrap_or_else(|e| e.into_inner());
        let entry = entries.get(&node.id)?;
        (entry.source == Some((node.node_type, node.pid)))
            .then(|| entry.sync.clone())
            .flatten()
    }

    /// Invalidate immediately on a managed launch, including PID reuse. The
    /// collector keeps its byte offset but may not republish an old sync sample.
    pub(crate) fn invalidate(&self, node_id: &str) {
        let mut entries = self.0.entries.write().unwrap_or_else(|e| e.into_inner());
        let entry = entries.entry(node_id.to_string()).or_default();
        entry.epoch = entry.epoch.wrapping_add(1);
        entry.sync = None;
    }

    pub(crate) fn epoch(&self, node_id: &str) -> u64 {
        self.0
            .entries
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .get(node_id)
            .map_or(0, |entry| entry.epoch)
    }

    pub(crate) fn publish(&self, node: &NodeConfig, epoch: u64, sync: Option<Arc<SyncProgress>>) {
        let mut entries = self.0.entries.write().unwrap_or_else(|e| e.into_inner());
        let entry = entries.entry(node.id.clone()).or_default();
        if entry.epoch == epoch {
            entry.source = Some((node.node_type, node.pid));
            entry.sync = sync;
        }
    }

    pub(crate) fn retain(&self, nodes: &[NodeConfig]) {
        self.0
            .entries
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .retain(|id, _| nodes.iter().any(|node| &node.id == id));
    }

    pub(crate) fn claim(&self) -> anyhow::Result<CollectionLease> {
        anyhow::ensure!(
            self.0
                .collecting
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .is_ok(),
            "the Engine already owns this workspace log collector"
        );
        self.0
            .entries
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
        Ok(CollectionLease(self.clone()))
    }
}

/// Only Engine acquires a lease; it moves the lease into its joined worker.
pub(crate) struct CollectionLease(LogObservations);

impl Drop for CollectionLease {
    fn drop(&mut self) {
        self.0
             .0
            .entries
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
        self.0 .0.collecting.store(false, Ordering::Release);
    }
}
