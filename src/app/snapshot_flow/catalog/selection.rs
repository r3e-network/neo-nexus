use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn snapshot_catalog_entry_filter(&self) -> SnapshotCatalogEntryFilter {
        SnapshotCatalogEntryFilter::new(
            self.snapshot_catalog_network_filter,
            self.snapshot_catalog_type_filter,
            self.snapshot_catalog_query.as_str(),
        )
    }

    pub(in crate::app) fn filtered_snapshot_catalog_entries(
        &self,
        entries: &[FastSyncSnapshotCatalogEntry],
    ) -> Vec<FastSyncSnapshotCatalogEntry> {
        filter_snapshot_catalog_entries(entries, &self.snapshot_catalog_entry_filter())
    }

    pub(in crate::app) fn ensure_valid_snapshot_catalog_selection(&mut self) {
        let Some(catalog) = &self.snapshot_catalog else {
            self.selected_snapshot_catalog_entry = None;
            self.snapshot_catalog_page = 0;
            return;
        };
        let entries = catalog.snapshots.clone();
        let visible = self.filtered_snapshot_catalog_entries(&entries);

        let selected_exists = self
            .selected_snapshot_catalog_entry
            .as_ref()
            .is_some_and(|id| visible.iter().any(|entry| &entry.id == id));
        if !selected_exists {
            self.selected_snapshot_catalog_entry = visible.first().map(|entry| entry.id.clone());
            self.snapshot_catalog_page = 0;
        }

        self.snapshot_catalog_page = clamp_page(
            self.snapshot_catalog_page,
            visible.len(),
            SNAPSHOT_CATALOG_PAGE_SIZE,
        );
    }

    pub(in crate::app) fn selected_snapshot_catalog_entry(
        &self,
    ) -> Option<FastSyncSnapshotCatalogEntry> {
        let selected_id = self.selected_snapshot_catalog_entry.as_deref()?;
        self.snapshot_catalog.as_ref()?.get(selected_id).cloned()
    }
}
