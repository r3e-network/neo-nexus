use eframe::egui;

use crate::app::domain::FastSyncSnapshot;

use super::super::super::{
    paging::page_count,
    text::truncate_middle,
    widgets::{empty_state, grid_header, pagination_bar},
    NeoNexusApp, SNAPSHOT_PAGE_SIZE,
};

use super::filter::render_snapshot_registry_filter;
use super::status::status_label;

impl NeoNexusApp {
    pub(super) fn render_snapshot_registry(
        &mut self,
        ui: &mut egui::Ui,
        snapshots: &[FastSyncSnapshot],
    ) {
        if snapshots.is_empty() {
            empty_state(
                ui,
                "No snapshots",
                "Register a local fast sync snapshot manifest first.",
            );
            return;
        }

        render_snapshot_registry_filter(self, ui);
        self.ensure_valid_snapshot_selection(snapshots);
        let filtered = self.filtered_snapshots(snapshots);
        if filtered.is_empty() {
            empty_state(ui, "No matching snapshots", "Adjust the snapshot filter.");
            return;
        }

        let total_pages = page_count(filtered.len(), SNAPSHOT_PAGE_SIZE);
        self.snapshot_page = self.snapshot_page.min(total_pages - 1);
        pagination_bar(ui, &mut self.snapshot_page, total_pages, filtered.len());
        ui.separator();

        let start = self.snapshot_page * SNAPSHOT_PAGE_SIZE;
        let visible = filtered.iter().skip(start).take(SNAPSHOT_PAGE_SIZE);

        egui::Grid::new("snapshot_registry")
            .striped(true)
            .min_col_width(72.0)
            .show(ui, |ui| {
                grid_header(ui, &["Label", "Runtime", "Network", "Status"]);

                for snapshot in visible {
                    let selected = self.selected_snapshot.as_deref() == Some(snapshot.id.as_str());
                    if ui
                        .selectable_label(selected, truncate_middle(&snapshot.label, 24))
                        .clicked()
                    {
                        self.selected_snapshot = Some(snapshot.id.clone());
                    }
                    ui.label(snapshot.node_type.to_string());
                    ui.label(snapshot.network.to_string());
                    ui.label(status_label(snapshot));
                    ui.end_row();
                }
            });
    }
}
