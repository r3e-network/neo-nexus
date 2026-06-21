use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn cache_selected_snapshot(&mut self) {
        let Some(snapshot) = self.selected_fast_sync_snapshot() else {
            self.notice = Some("Select a fast sync snapshot before caching it".to_string());
            return;
        };

        match FastSyncSnapshotManager::cache(&snapshot, self.snapshot_cache_dir()) {
            Ok(cache) => {
                if let Err(error) = self
                    .repository
                    .mark_fast_sync_snapshot_cached(&snapshot.id, &cache)
                {
                    self.notice = Some(error.to_string());
                    return;
                }

                let message = format!(
                    "{} cached: {} bytes at {}",
                    snapshot.label,
                    cache.bytes,
                    short_path(&cache.path, 54)
                );
                self.record_event_notice(EventKind::SnapshotCached, EventSeverity::Info, message);
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }
}
