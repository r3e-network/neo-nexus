use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn load_selected_snapshot_catalog_entry_into_draft(&mut self) {
        let Some(entry) = self.selected_snapshot_catalog_entry() else {
            self.notice = Some("Select a fast sync catalog entry first".to_string());
            return;
        };

        self.load_snapshot_catalog_entry_into_draft(&entry);
        self.notice = Some(format!("Catalog snapshot loaded: {}", entry.label));
    }

    pub(in crate::app) fn save_selected_snapshot_catalog_entry_manifest(&mut self) {
        let Some(entry) = self.selected_snapshot_catalog_entry() else {
            self.notice = Some("Select a fast sync catalog entry first".to_string());
            return;
        };

        match self
            .repository
            .upsert_fast_sync_snapshot(entry.to_new_snapshot())
        {
            Ok(snapshot) => {
                self.selected_snapshot = Some(snapshot.id.clone());
                self.load_snapshot_catalog_entry_into_draft(&entry);
                let message = format!("Fast sync catalog snapshot saved: {}", snapshot.label);
                self.record_event(
                    None,
                    None,
                    EventKind::SnapshotSaved,
                    EventSeverity::Info,
                    message.clone(),
                );
                self.notice = Some(message);
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn download_selected_snapshot_catalog_entry(&mut self) {
        let Some(entry) = self.selected_snapshot_catalog_entry() else {
            self.notice = Some("Select a fast sync catalog entry first".to_string());
            return;
        };

        match self
            .repository
            .upsert_fast_sync_snapshot(entry.to_new_snapshot())
            .and_then(|snapshot| self.download_fast_sync_snapshot(&snapshot))
        {
            Ok(message) => {
                self.load_snapshot_catalog_entry_into_draft(&entry);
                self.notice = Some(message);
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }
}
