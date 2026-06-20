use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn ensure_valid_snapshot_selection(
        &mut self,
        snapshots: &[FastSyncSnapshot],
    ) {
        let visible = self.filtered_snapshots(snapshots);
        let selected_exists = self
            .selected_snapshot
            .as_ref()
            .is_some_and(|id| visible.iter().any(|snapshot| &snapshot.id == id));
        if !selected_exists {
            self.selected_snapshot = visible.first().map(|snapshot| snapshot.id.clone());
            self.snapshot_page = 0;
        }
        self.snapshot_page = super::super::super::paging::clamp_page(
            self.snapshot_page,
            visible.len(),
            SNAPSHOT_PAGE_SIZE,
        );
    }
}
