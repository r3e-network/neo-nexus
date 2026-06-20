use eframe::egui;

use crate::app::NeoNexusApp;

pub(super) fn render_install_actions(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui.button("Download HTTPS").clicked() {
            app.download_runtime_package();
        }
        if ui.button("Install").clicked() {
            app.install_runtime_package();
        }
        if ui.button("Current Platform").clicked() {
            app.runtime_package_draft.use_current_platform();
        }
        if ui.button("Reset").clicked() {
            app.runtime_package_draft = Default::default();
        }
    });
}
