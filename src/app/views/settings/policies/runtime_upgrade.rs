mod actions;
mod form;
mod status;

use eframe::egui;

use super::super::super::super::NeoNexusApp;

impl NeoNexusApp {
    pub(in crate::app::views::settings) fn render_runtime_upgrade_policy_settings(
        &mut self,
        ui: &mut egui::Ui,
    ) {
        let active_policy = self.runtime_upgrade_policy.clone();

        form::render_policy_form(self, ui);
        actions::render_policy_actions(self, ui, &active_policy);
        status::render_policy_status(ui, &active_policy);
    }
}
