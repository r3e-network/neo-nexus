use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn load_fast_sync_snapshot_catalog(&mut self) {
        let request = SnapshotCatalogLoadRequest {
            source: self.snapshot_catalog_source.trim().to_string(),
            signature_source: optional_text(&self.snapshot_catalog_signature_source),
            ed25519_public_key: optional_text(&self.snapshot_catalog_public_key),
            max_bytes: FastSyncSnapshotManager::DEFAULT_CATALOG_MAX_BYTES,
        };

        match FastSyncSnapshotManager::load_catalog(&request) {
            Ok(load) => {
                let catalog = load.catalog;
                let count = catalog.snapshots.len();
                self.snapshot_catalog_page = 0;
                self.selected_snapshot_catalog_entry = catalog
                    .snapshots
                    .first()
                    .map(|snapshot| snapshot.id.clone());
                self.snapshot_catalog_signature_verified = load.signature_verified;
                self.snapshot_catalog_bytes = load.bytes;
                self.snapshot_catalog = Some(catalog);
                self.notice = Some(format!(
                    "Fast sync catalog loaded: {count} snapshots ({})",
                    format_bytes(load.bytes)
                ));
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }
}
