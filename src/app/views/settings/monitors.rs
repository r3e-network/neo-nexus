mod federation;
mod retention;
mod rpc;
mod summary;
mod widgets;

use eframe::egui;

use super::super::super::NeoNexusApp;

impl NeoNexusApp {
    pub(super) fn render_rpc_monitor_settings(&mut self, ui: &mut egui::Ui) {
        summary::render_monitor_summary(self, ui);
        ui.separator();
        rpc::render_rpc_monitor_policy(self, ui);
        ui.separator();
        federation::render_federation_monitor_policy(self, ui);
        ui.add_space(4.0);
        retention::render_monitor_retention(self, ui);
    }
}
