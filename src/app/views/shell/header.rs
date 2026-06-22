mod actions;
mod menu;
mod state;

use eframe::egui;

use self::{actions::render_node_action_buttons, menu::render_application_menu};
use super::super::super::{theme::muted_text, NeoNexusApp};

// Re-exported for the menu submodule's `use super::View`.
pub(super) use super::super::super::view::View;

impl NeoNexusApp {
    pub(in crate::app) fn render_application_header(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(2.0);
            ui.vertical(|ui| {
                ui.add_space(1.0);
                ui.label(
                    egui::RichText::new(self.selected_view.title())
                        .strong()
                        .size(17.0),
                );
                ui.label(
                    egui::RichText::new(self.selected_view.subtitle())
                        .color(muted_text())
                        .size(12.0),
                );
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(6.0);
                render_node_action_buttons(self, ui);
                ui.separator();
                render_application_menu(self, ui);
            });
        });
    }
}
