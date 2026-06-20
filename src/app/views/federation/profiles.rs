mod actions;
mod contract;
mod filter;
mod form;
mod list;

use eframe::egui;

use crate::app::NeoNexusApp;

impl NeoNexusApp {
    pub(in crate::app::views::federation) fn render_remote_profile_list(
        &mut self,
        ui: &mut egui::Ui,
    ) {
        list::render_remote_profile_list(self, ui);
    }

    pub(in crate::app::views::federation) fn render_remote_profile_editor(
        &mut self,
        ui: &mut egui::Ui,
    ) {
        form::render_remote_profile_form(self, ui);
        actions::render_remote_profile_actions(self, ui);
        contract::render_public_endpoint_contract(ui);
    }
}
