mod definition;
mod layout;
mod selected;

use eframe::egui;

use super::super::{widgets::panel, NeoNexusApp};

impl NeoNexusApp {
    pub(super) fn render_nodes(&mut self, ui: &mut egui::Ui) {
        let layout = layout::node_pane_layout(ui.available_size());

        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(layout.definition_width, layout.height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    panel(ui, "Node definition", |ui| {
                        definition::render_create_form(self, ui);
                    });
                },
            );

            ui.add_space(layout.gap);

            ui.allocate_ui_with_layout(
                egui::vec2(layout.selected_width, layout.height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    panel(ui, "Selected node", |ui| {
                        selected::render_selected_node_editor(self, ui);
                    });
                },
            );
        });
    }
}
