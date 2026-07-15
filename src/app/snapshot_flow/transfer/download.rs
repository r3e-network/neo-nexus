use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn download_selected_snapshot(&mut self) {
        let Some(snapshot) = self.selected_fast_sync_snapshot() else {
            self.session.notice = Some("Select a fast sync snapshot before downloading it".to_string());
            return;
        };

        match self.download_fast_sync_snapshot(&snapshot) {
            Ok(message) => self.session.notice = Some(message),
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn download_fast_sync_snapshot(
        &mut self,
        snapshot: &FastSyncSnapshot,
    ) -> anyhow::Result<String> {
        let request = snapshot.download_request()?;
        let cache = FastSyncSnapshotManager::download_https(&request, self.snapshot_cache_dir())?;
        self.repository
            .mark_fast_sync_snapshot_cached(&snapshot.id, &cache)?;
        self.selected_snapshot = Some(snapshot.id.clone());
        let message = format!(
            "{} downloaded and cached: {} bytes at {}",
            snapshot.label,
            cache.bytes,
            short_path(&cache.path, 54)
        );
        self.record_event(
            None,
            None,
            EventKind::SnapshotDownloaded,
            EventSeverity::Info,
            message.clone(),
        );
        Ok(message)
    }
}
