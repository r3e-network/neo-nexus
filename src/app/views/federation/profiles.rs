mod actions;
mod contract;
mod filter;
mod form;
mod list;

use eframe::egui;

use crate::app::NeoNexusApp;

impl NeoNexusApp {
    /// How many profiles there are and how many are probing, above the list
    /// they count.
    ///
    /// These were three tiles in a page-level metric row, restated on the
    /// Editor, Inspector and Governance surfaces where they say nothing about
    /// what is on screen.
    pub(in crate::app::views::federation) fn render_remote_profile_counts(
        &self,
        ui: &mut egui::Ui,
    ) {
        let total = self.remote_servers.len();
        let enabled = self
            .remote_servers
            .iter()
            .filter(|profile| profile.enabled)
            .count();
        let summary = if total == 0 {
            "No remote profiles".to_string()
        } else {
            format!("{total} profiles · {enabled} probing")
        };
        ui.label(egui::RichText::new(summary).color(crate::app::theme::muted_text()));
    }

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
