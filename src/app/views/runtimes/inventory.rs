use eframe::egui;

use crate::app::domain::{format_bytes, RuntimeInstallation};

use super::super::super::{
    paging::page_count,
    text::truncate_middle,
    theme,
    widgets::{empty_state, empty_state_with_action, grid_header, pagination_bar, text_badge},
    NeoNexusApp, RUNTIME_PAGE_SIZE,
};
use super::filter::render_runtime_inventory_filter;
use super::section::RuntimesSection;

impl NeoNexusApp {
    pub(super) fn render_runtime_inventory(
        &mut self,
        ui: &mut egui::Ui,
        installations: &[RuntimeInstallation],
    ) {
        if installations.is_empty() {
            if empty_state_with_action(
                ui,
                "No runtimes",
                "Install a verified local runtime package to populate inventory.",
                Some("Go to Install"),
            ) {
                self.sections.runtimes = RuntimesSection::Install;
            }
            return;
        }

        render_runtime_inventory_filter(self, ui);
        self.ensure_valid_runtime_selection(installations);
        let filtered = self.filtered_runtime_installations(installations);
        if filtered.is_empty() {
            empty_state(ui, "No matching runtimes", "Adjust the runtime filter.");
            return;
        }

        let total_pages = page_count(filtered.len(), RUNTIME_PAGE_SIZE);
        self.runtime_page = self.runtime_page.min(total_pages - 1);
        pagination_bar(ui, &mut self.runtime_page, total_pages, filtered.len());
        ui.add_space(theme::SM);

        let start = self.runtime_page * RUNTIME_PAGE_SIZE;
        egui::Grid::new("runtime_inventory")
            .striped(true)
            .min_col_width(74.0)
            .show(ui, |ui| {
                grid_header(
                    ui,
                    &["Package", "Runtime", "Version", "Platform", "Trust", "Size"],
                );

                for installation in filtered.iter().skip(start).take(RUNTIME_PAGE_SIZE) {
                    let selected = self.selected_runtime_installation.as_deref()
                        == Some(installation.package_id.as_str());
                    if ui
                        .selectable_label(selected, truncate_middle(&installation.label, 22))
                        .clicked()
                    {
                        self.selected_runtime_installation = Some(installation.package_id.clone());
                    }
                    ui.label(installation.node_type.to_string());
                    ui.label(truncate_middle(&installation.version, 16));
                    ui.label(installation.platform.to_string());
                    if installation.signature_verified {
                        text_badge(ui, "signed", theme::success());
                    } else {
                        text_badge(ui, "hash", theme::muted_text());
                    }
                    ui.label(format_bytes(installation.bytes));
                    ui.end_row();
                }
            });
    }
}
