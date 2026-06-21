use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn apply_selected_snapshot_to_node(&mut self) {
        let Some(node) = self.selected_node().cloned() else {
            self.notice = Some("Select a node before applying a fast sync snapshot".to_string());
            return;
        };
        if node.status.is_active() {
            self.notice =
                Some("Stop the selected node before applying a fast sync snapshot".to_string());
            return;
        }

        let Some(snapshot) = self.selected_fast_sync_snapshot() else {
            self.notice = Some("Select a fast sync snapshot before applying it".to_string());
            return;
        };

        match FastSyncSnapshotManager::apply_to_node(&snapshot, &node, self.node_data_dir(&node)) {
            Ok(application) => {
                let message = format!(
                    "{} imported to {} data dir via {}: {} files, {} bytes at {}",
                    snapshot.label,
                    node.name,
                    application.import_mode.label(),
                    application.imported_files,
                    application.expanded_bytes,
                    short_path(&application.snapshot_path, 54)
                );
                self.record_node_event(
                    &node,
                    EventKind::SnapshotApplied,
                    EventSeverity::Info,
                    message.clone(),
                );
                self.notice = Some(message);
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }
}
