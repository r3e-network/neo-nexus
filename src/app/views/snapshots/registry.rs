use eframe::egui;

use crate::app::domain::FastSyncSnapshot;

use super::super::super::{
    paging::{page_count, rows_that_fit},
    text::truncate_middle,
    widgets::{empty_state, grid_header, pagination_bar},
    NeoNexusApp, SNAPSHOT_PAGE_SIZE,
};

use super::filter::render_snapshot_registry_filter;
use super::status::status_label;

/// One registry grid row: a `selectable_label` plus the grid's row gap.
/// Measured at Comfortable density; Compact rows are shorter, so the derived
/// page size stays on the safe side of the panel edge there.
const ROW_HEIGHT: f32 = 39.0;

/// Drawn between the height probe and the first data row: the pagination bar,
/// the separator under it, and the grid's own column header row.
const CHROME_HEIGHT: f32 = 89.0;

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

        // Probed after the filter block so its height is already spent; the
        // chrome still to come below the probe is subtracted instead.
        let page_size =
            rows_that_fit(ui.available_height(), ROW_HEIGHT, CHROME_HEIGHT).min(SNAPSHOT_PAGE_SIZE);
        let total_pages = page_count(filtered.len(), page_size);
        self.snapshot_page = self.snapshot_page.min(total_pages - 1);
        pagination_bar(ui, &mut self.snapshot_page, total_pages, filtered.len());
        ui.separator();

        let start = self.snapshot_page * page_size;
        let visible = filtered.iter().skip(start).take(page_size);

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
