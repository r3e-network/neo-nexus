use eframe::egui;

use super::{actions::render_node_action_buttons, NeoNexusApp};

impl NeoNexusApp {
    pub(super) fn render_application_navigation_row(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            render_node_action_buttons(self, ui);
        });
    }
}
