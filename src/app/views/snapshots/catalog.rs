use eframe::egui;

use crate::app::domain::format_bytes;

use super::super::super::{
    paging::page_count,
    text::truncate_middle,
    widgets::{empty_state, fact, labeled_text, pagination_bar},
    NeoNexusApp, SNAPSHOT_CATALOG_PAGE_SIZE,
};
use super::filter::render_snapshot_catalog_filter;

impl NeoNexusApp {
    pub(super) fn render_snapshot_catalog(&mut self, ui: &mut egui::Ui) {
        labeled_text(ui, "Catalog", &mut self.snapshot_catalog_source);
        labeled_text(ui, "Signature", &mut self.snapshot_catalog_signature_source);
        labeled_text(ui, "Public key", &mut self.snapshot_catalog_public_key);
        ui.horizontal(|ui| {
            if ui.button("Load").clicked() {
                self.load_fast_sync_snapshot_catalog();
            }
            if ui.button("Use").clicked() {
                self.load_selected_snapshot_catalog_entry_into_draft();
            }
            if ui.button("Save").clicked() {
                self.save_selected_snapshot_catalog_entry_manifest();
            }
            if ui.button("Download").clicked() {
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

        let total_pages = page_count(filtered.len(), SNAPSHOT_CATALOG_PAGE_SIZE);
        self.snapshot_catalog_page = self.snapshot_catalog_page.min(total_pages - 1);
        pagination_bar(
            ui,
            &mut self.snapshot_catalog_page,
            total_pages,
            filtered.len(),
        );
        ui.separator();

        let start = self.snapshot_catalog_page * SNAPSHOT_CATALOG_PAGE_SIZE;
        egui::Grid::new("snapshot_catalog_entries")
            .striped(true)
            .min_col_width(66.0)
            .show(ui, |ui| {
                ui.strong("Snapshot");
                ui.strong("Runtime");
                ui.strong("Network");
                ui.strong("Limit");
                ui.end_row();

                for entry in filtered.iter().skip(start).take(SNAPSHOT_CATALOG_PAGE_SIZE) {
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
