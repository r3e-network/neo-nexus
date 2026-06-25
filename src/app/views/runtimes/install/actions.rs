use eframe::egui;

use crate::app::{
    widgets::{primary_button, secondary_button},
    NeoNexusApp,
};

pub(super) fn render_install_actions(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if secondary_button(ui, "Download HTTPS").clicked() {
            app.download_runtime_package();
        }
        if primary_button(ui, "Install").clicked() {
            app.install_runtime_package();
        }
        if secondary_button(ui, "Current Platform").clicked() {
            app.runtime_package_draft.use_current_platform();
        }
        if secondary_button(ui, "Reset").clicked() {
            app.runtime_package_draft = Default::default();
        }
    });
}
