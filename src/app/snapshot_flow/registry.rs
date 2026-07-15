use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn snapshot_filter(&self) -> SnapshotFilter {
        SnapshotFilter::new(
            self.snapshot_network_filter,
            self.snapshot_type_filter,
            self.snapshot_verified_filter,
            self.snapshot_cached_filter,
            self.snapshot_query.as_str(),
        )
    }

    pub(in crate::app) fn filtered_snapshots(
        &self,
        snapshots: &[FastSyncSnapshot],
    ) -> Vec<FastSyncSnapshot> {
        filter_snapshots(snapshots, &self.snapshot_filter())
    }

    pub(in crate::app) fn save_snapshot_manifest(&mut self) {
        let input = match self.snapshot_draft.to_new_snapshot() {
            Ok(input) => input,
            Err(error) => {
                self.session.notice = Some(error.to_string());
                return;
            }
        };

        match self.repository.upsert_fast_sync_snapshot(input) {
            Ok(snapshot) => {
                self.selected_snapshot = Some(snapshot.id.clone());
                let message = format!("Fast sync snapshot saved: {}", snapshot.label);
                self.record_event_notice(EventKind::SnapshotSaved, EventSeverity::Info, message);
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn download_snapshot_from_draft(&mut self) {
        let input = match self.snapshot_draft.to_new_snapshot() {
            Ok(input) => input,
            Err(error) => {
                self.session.notice = Some(error.to_string());
                return;
            }
        };

        match self
            .repository
            .upsert_fast_sync_snapshot(input)
            .and_then(|snapshot| self.download_fast_sync_snapshot(&snapshot))
        {
            Ok(message) => self.session.notice = Some(message),
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }
}
