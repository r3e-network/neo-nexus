use eframe::egui;

use super::super::super::NeoNexusApp;

mod actions;
mod fields;
mod status;
mod summary;

impl NeoNexusApp {
    pub(super) fn render_runtime_install_form(&mut self, ui: &mut egui::Ui) {
        fields::render_package_fields(self, ui);
        ui.separator();
        fields::render_download_fields(self, ui);
        ui.add_space(8.0);
        status::render_manifest_status(self, ui);
        ui.add_space(8.0);
        actions::render_install_actions(self, ui);
        ui.separator();
        summary::render_install_summary(self, ui);
    }
}
