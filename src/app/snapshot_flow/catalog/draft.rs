use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn load_snapshot_catalog_entry_into_draft(
        &mut self,
        entry: &FastSyncSnapshotCatalogEntry,
    ) {
        self.snapshot_draft.id = entry.id.clone();
        self.snapshot_draft.label = entry.label.clone();
        self.snapshot_draft.network = entry.network;
        self.snapshot_draft.node_type = entry.node_type;
        self.snapshot_draft.source_path.clear();
        self.snapshot_draft.source_url = entry.url.clone();
        self.snapshot_draft.download_file_name = entry.file_name.clone();
        self.snapshot_draft.download_max_mib =
            entry.max_bytes.saturating_add(1024 * 1024 - 1) / (1024 * 1024);
        self.snapshot_draft.expected_sha256 = entry.expected_sha256.clone();
    }
}
