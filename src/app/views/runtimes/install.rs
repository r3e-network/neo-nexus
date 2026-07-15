use eframe::egui;

use super::super::super::{theme, NeoNexusApp};

mod actions;
mod fields;
mod status;
mod summary;

impl NeoNexusApp {
    pub(super) fn render_runtime_install_form(&mut self, ui: &mut egui::Ui) {
        // Two-column workbench: identity/source/integrity left, download +
        // validation + actions + paths right — keeps the install wizard scannable.
        let gap = theme::MD;
        let width = ui.available_width();
        let left = (width * 0.54).clamp(width * 0.45, width * 0.62);
        let right = (width - left - gap).max(width * 0.34);

        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(left, ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    fields::render_package_fields(self, ui);
                },
            );
            ui.add_space(gap);
            ui.allocate_ui_with_layout(
                egui::vec2(right, ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    fields::render_download_fields(self, ui);
                    ui.add_space(theme::MD);
                    status::render_manifest_status(self, ui);
                    ui.add_space(theme::MD);
                    actions::render_install_actions(self, ui);
                    ui.add_space(theme::MD);
                    ui.separator();
                    ui.add_space(theme::SM);
                    summary::render_install_summary(self, ui);
                },
            );
        });
    }
}
