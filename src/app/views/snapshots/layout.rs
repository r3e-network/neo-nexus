use super::*;

impl NeoNexusApp {
    pub(in crate::app::views) fn render_snapshots(&mut self, ui: &mut egui::Ui) {
        let snapshots = match self.repository.list_fast_sync_snapshots() {
            Ok(snapshots) => snapshots,
            Err(error) => {
                empty_state(ui, "Snapshot registry unavailable", &error.to_string());
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

        ui.horizontal(|ui| {
            metric_tile(ui, "Snapshots", &snapshots.len().to_string(), "registered");
            metric_tile(ui, "Verified", &verified.to_string(), "sha-256 match");
            metric_tile(ui, "Cached", &cached.to_string(), "local files");
            metric_tile(
                ui,
                "Cache",
                &short_path(&self.snapshot_cache_dir(), 28),
                "workspace",
            );
        });
    }

    fn render_snapshot_workspace(&mut self, ui: &mut egui::Ui, snapshots: &[FastSyncSnapshot]) {
        ui.add_space(10.0);
        let available = ui.available_size();
        let top_height = (available.y * 0.62).clamp(340.0, 460.0);
        let manifest_width = (available.x * 0.35).clamp(340.0, 470.0);
        let catalog_width = (available.x * 0.32).clamp(320.0, 450.0);

        ui.horizontal(|ui| {
            self.render_snapshot_manifest_panel(ui, manifest_width, top_height);
            ui.add_space(8.0);
            self.render_snapshot_catalog_panel(ui, catalog_width, top_height);
            ui.add_space(8.0);
            self.render_snapshot_registry_panel(
                ui,
                (available.x - manifest_width - catalog_width - 16.0).max(320.0),
                top_height,
                snapshots,
            );
        });

        self.render_snapshot_actions_panel(ui, snapshots);
    }

    fn render_snapshot_manifest_panel(&mut self, ui: &mut egui::Ui, width: f32, height: f32) {
        ui.allocate_ui_with_layout(
            egui::vec2(width, height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                panel(ui, "Snapshot manifest", |ui| {
                    self.render_snapshot_manifest_form(ui);
                });
            },
        );
    }

    fn render_snapshot_catalog_panel(&mut self, ui: &mut egui::Ui, width: f32, height: f32) {
        ui.allocate_ui_with_layout(
            egui::vec2(width, height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                panel(ui, "Snapshot catalog", |ui| {
                    self.render_snapshot_catalog(ui);
                });
            },
        );
    }

    fn render_snapshot_registry_panel(
        &mut self,
        ui: &mut egui::Ui,
        width: f32,
        height: f32,
        snapshots: &[FastSyncSnapshot],
    ) {
        ui.allocate_ui_with_layout(
            egui::vec2(width, height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                panel(ui, "Registry", |ui| {
                    self.render_snapshot_registry(ui, snapshots);
                });
            },
        );
    }

    fn render_snapshot_actions_panel(&mut self, ui: &mut egui::Ui, snapshots: &[FastSyncSnapshot]) {
        ui.add_space(8.0);
        let bottom = ui.available_size();
        ui.allocate_ui_with_layout(
            egui::vec2(bottom.x, bottom.y),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                panel(ui, "Verification and cache", |ui| {
                    self.render_selected_snapshot_actions(ui, snapshots);
                });
            },
        );
    }
}
