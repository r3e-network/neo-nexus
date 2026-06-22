use eframe::egui;

use crate::app::domain::{format_bytes, RuntimePlatform, RuntimeRelease};

use super::super::super::super::{
    paging::page_count,
    text::truncate_middle,
    theme::{accent, muted_text},
    widgets::{empty_state, grid_header, pagination_bar},
    NeoNexusApp, RUNTIME_CATALOG_PAGE_SIZE,
};
use super::super::filter::render_runtime_release_filter;

impl NeoNexusApp {
    pub(super) fn render_runtime_catalog_table(
        &mut self,
        ui: &mut egui::Ui,
        releases: &[RuntimeRelease],
        platform: &RuntimePlatform,
    ) {
        if releases.is_empty() {
            empty_state(ui, "Empty catalog", "No runtime releases were listed.");
            return;
        }

        render_runtime_release_filter(self, ui);
        self.ensure_valid_runtime_release_selection();
        let filtered = self.filtered_runtime_releases(releases, platform);
        if filtered.is_empty() {
            empty_state(ui, "No matching releases", "Adjust the release filter.");
            return;
        }

        let total_pages = page_count(filtered.len(), RUNTIME_CATALOG_PAGE_SIZE);
        self.runtime_catalog_page = self.runtime_catalog_page.min(total_pages - 1);
        pagination_bar(
            ui,
            &mut self.runtime_catalog_page,
            total_pages,
            filtered.len(),
        );
        ui.separator();

        let start = self.runtime_catalog_page * RUNTIME_CATALOG_PAGE_SIZE;
        egui::Grid::new("runtime_release_catalog")
            .striped(true)
            .min_col_width(72.0)
            .show(ui, |ui| {
                grid_header(ui, &["Release", "Runtime", "Version", "Platform", "Limit"]);

                for release in filtered.iter().skip(start).take(RUNTIME_CATALOG_PAGE_SIZE) {
                    let selected =
                        self.selected_runtime_release.as_deref() == Some(release.id.as_str());
                    let compatible = release.platform == *platform;
                    let text = egui::RichText::new(truncate_middle(&release.label, 22))
                        .color(if compatible { accent() } else { muted_text() });
                    if ui.selectable_label(selected, text).clicked() {
                        self.selected_runtime_release = Some(release.id.clone());
                    }
                    ui.label(release.node_type.to_string());
                    ui.label(truncate_middle(&release.version, 14));
                    ui.label(release.platform.to_string());
                    ui.label(format_bytes(release.max_bytes));
                    ui.end_row();
                }
            });
    }
}
