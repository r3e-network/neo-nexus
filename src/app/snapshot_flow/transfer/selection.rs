use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn selected_fast_sync_snapshot(&self) -> Option<FastSyncSnapshot> {
        let selected_id = self.selected_snapshot.as_deref()?;
        self.repository
            .list_fast_sync_snapshots()
            .ok()?
            .into_iter()
            .find(|snapshot| snapshot.id == selected_id)
    }
}
