mod actions;
mod menu;
mod state;

use eframe::egui;

use self::{actions::render_node_action_buttons, menu::render_application_menu};
use super::super::super::{theme, NeoNexusApp};

// Re-exported for the menu submodule's `use super::View`.
pub(super) use super::super::super::view::View;

impl NeoNexusApp {
    pub(in crate::app) fn render_application_header(&mut self, ui: &mut egui::Ui) {
        ui.add_space(theme::SM);
        ui.horizontal(|ui| {
            ui.add_space(theme::XS);
            ui.vertical(|ui| {
                ui.add_space(1.0);
                ui.label(theme::page_title(self.session.selected_view.title()));
                ui.label(theme::muted_body(self.session.selected_view.subtitle()));
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(theme::SM);
                render_node_action_buttons(self, ui);
                ui.separator();
                render_application_menu(self, ui);
            });
        });
    }
}
