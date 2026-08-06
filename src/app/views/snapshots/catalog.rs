use eframe::egui;

use crate::app::domain::format_bytes;

use super::super::super::{
    paging::{page_count, rows_that_fit},
    text::truncate_middle,
    widgets::{empty_state, fact, grid_header, labeled_text, pagination_bar, secondary_button},
    NeoNexusApp, SNAPSHOT_CATALOG_PAGE_SIZE,
};
use super::filter::render_snapshot_catalog_filter;

/// One catalog grid row: a `selectable_label` plus the grid's row gap.
/// Measured at Comfortable density; Compact rows are shorter, so the derived
/// page size stays on the safe side of the panel edge there.
const ROW_HEIGHT: f32 = 39.0;

/// Drawn between the height probe and the first data row: the pagination bar,
/// the separator under it, and the grid's own column header row.
const CHROME_HEIGHT: f32 = 89.0;

impl NeoNexusApp {
    pub(super) fn render_snapshot_catalog(&mut self, ui: &mut egui::Ui) {
        labeled_text(ui, "Catalog", &mut self.snapshot_catalog_source);
        labeled_text(ui, "Signature", &mut self.snapshot_catalog_signature_source);
        labeled_text(ui, "Public key", &mut self.snapshot_catalog_public_key);
        ui.horizontal(|ui| {
            if secondary_button(ui, "Load").clicked() {
                self.load_fast_sync_snapshot_catalog();
            }
            if secondary_button(ui, "Use").clicked() {
                self.load_selected_snapshot_catalog_entry_into_draft();
            }
            if secondary_button(ui, "Save").clicked() {
                self.save_selected_snapshot_catalog_entry_manifest();
            }
            if secondary_button(ui, "Download").clicked() {
                self.download_selected_snapshot_catalog_entry();
            }
        });
        ui.separator();

        let Some(catalog) = &self.snapshot_catalog else {
            empty_state(
                ui,
                "No catalog",
                "Load a local or signed HTTPS fast sync catalog.",
            );
            return;
        };

        let entries = catalog.snapshots.clone();
        let trust = if self.snapshot_catalog_signature_verified == Some(true) {
            "signed"
        } else {
            "local"
        };
        ui.columns(2, |columns| {
            fact(&mut columns[0], "Entries", &entries.len().to_string());
            fact(&mut columns[0], "Trust", trust);
            fact(
                &mut columns[1],
                "Size",
                &format_bytes(self.snapshot_catalog_bytes),
            );
            fact(
                &mut columns[1],
                "Generated",
                &catalog
                    .generated_at_unix
                    .map_or_else(|| "unknown".to_string(), |value| value.to_string()),
            );
        });

        if entries.is_empty() {
            empty_state(ui, "Empty catalog", "No fast sync snapshots were listed.");
            return;
        }

        render_snapshot_catalog_filter(self, ui);
        self.ensure_valid_snapshot_catalog_selection();
        let filtered = self.filtered_snapshot_catalog_entries(&entries);
        if filtered.is_empty() {
            empty_state(
                ui,
                "No matching entries",
                "Adjust the snapshot catalog filter.",
            );
            return;
        }

        // Probed after the catalog filter so its height is already spent; the
        // chrome still to come below the probe is subtracted instead.
        let page_size = rows_that_fit(ui.available_height(), ROW_HEIGHT, CHROME_HEIGHT)
            .min(SNAPSHOT_CATALOG_PAGE_SIZE);
        let total_pages = page_count(filtered.len(), page_size);
        self.snapshot_catalog_page = self.snapshot_catalog_page.min(total_pages - 1);
        pagination_bar(
            ui,
            &mut self.snapshot_catalog_page,
            total_pages,
            filtered.len(),
        );
        ui.separator();

        let start = self.snapshot_catalog_page * page_size;
        egui::Grid::new("snapshot_catalog_entries")
            .striped(true)
            .min_col_width(66.0)
            .show(ui, |ui| {
                grid_header(ui, &["Snapshot", "Runtime", "Network", "Limit"]);

                for entry in filtered.iter().skip(start).take(page_size) {
                    let selected =
                        self.selected_snapshot_catalog_entry.as_deref() == Some(entry.id.as_str());
                    if ui
                        .selectable_label(selected, truncate_middle(&entry.label, 20))
                        .clicked()
                    {
                        self.selected_snapshot_catalog_entry = Some(entry.id.clone());
                    }
                    ui.label(entry.node_type.to_string());
                    ui.label(entry.network.to_string());
                    ui.label(format_bytes(entry.max_bytes));
                    ui.end_row();
                }
            });
    }
}
