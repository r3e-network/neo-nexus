use super::*;

impl NeoNexusApp {
    pub(in crate::app::views) fn render_snapshots(&mut self, ui: &mut egui::Ui) {
        let snapshots = match list_workspace_snapshots(&self.repository) {
            Ok(snapshots) => snapshots,
            Err(error) => {
                empty_state(ui, "Snapshot registry unavailable", "The snapshot registry could not be loaded. Reloading the workspace may resolve transient access errors.");
                ui.add_space(theme::SM);
                // Surface the technical cause as a small danger-coloured detail
                // below the empty state (errors must use a semantic colour per the
                // visual contract), rather than as the primary centred message so a
                // long anyhow trace doesn't blow out the layout.
                ui.vertical_centered(|ui| {
                    ui.label(
                        theme::label_caption(truncate_middle(&error.to_string(), 72))
                            .color(theme::danger()),
                    );
                });
                return;
            }
        };

        self.ensure_valid_snapshot_selection(&snapshots);
        self.ensure_valid_snapshot_catalog_selection();
        self.render_snapshot_metrics(ui, &snapshots);
        self.render_snapshot_workspace(ui, &snapshots);
    }

    fn render_snapshot_metrics(&self, ui: &mut egui::Ui, snapshots: &[FastSyncSnapshot]) {
        let verified = snapshots
            .iter()
            .filter(|snapshot| snapshot_is_verified(snapshot))
            .count();
        let cached = snapshots
            .iter()
            .filter(|snapshot| snapshot.cached_path.is_some())
            .count();

        let total = snapshots.len().to_string();
        let verified_label = verified.to_string();
        let cached_label = cached.to_string();
        let cache_dir = short_path(&self.snapshot_cache_dir(), 28);
        metric_row(
            ui,
            &[
                ("Snapshots", &total, "registered"),
                ("Verified", &verified_label, "sha-256 match"),
                ("Cached", &cached_label, "local files"),
                ("Cache", &cache_dir, "workspace"),
            ],
        );
    }

    fn render_snapshot_workspace(&mut self, ui: &mut egui::Ui, snapshots: &[FastSyncSnapshot]) {
        ui.add_space(theme::MD);
        let mut index = self.sections.snapshots as usize;
        let labels = SnapshotsSection::ALL.map(SnapshotsSection::label);
        if page_chrome(ui, None, Some((&labels, &mut index))) {
            self.sections.snapshots = SnapshotsSection::ALL[index];
        }

        match self.sections.snapshots {
            SnapshotsSection::Manifest => panel(ui, "Snapshot manifest", |ui| {
                self.render_snapshot_manifest_form(ui);
            }),
            SnapshotsSection::Catalog => panel(ui, "Snapshot catalog", |ui| {
                self.render_snapshot_catalog(ui);
            }),
            SnapshotsSection::Registry => panel(ui, "Registry", |ui| {
                self.render_snapshot_registry(ui, snapshots);
            }),
            SnapshotsSection::Verify => panel(ui, "Verification and cache", |ui| {
                self.render_selected_snapshot_actions(ui, snapshots);
            }),
        }
    }
}
