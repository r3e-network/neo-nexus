use eframe::egui;

use crate::{app::NeoNexusApp, runtime::RuntimeInstallation};

mod actions;
mod facts;
mod panel;
mod status;

impl NeoNexusApp {
    pub(super) fn render_runtime_application(
        &mut self,
        ui: &mut egui::Ui,
        installations: &[RuntimeInstallation],
    ) {
        panel::render_runtime_application(self, ui, installations);
    }
}
