mod details;
mod filter;
mod layout;
mod list;
mod package;

use eframe::egui;

use super::super::{
    view::View,
    widgets::{empty_state_with_action, panel},
    NeoNexusApp,
};

impl NeoNexusApp {
    pub(super) fn render_plugins(&mut self, ui: &mut egui::Ui) {
        let Some(node) = self.selected_node().cloned() else {
            let cta = if self.fleet.nodes.is_empty() {
                Some("Create node")
            } else {
                None
            };
            if empty_state_with_action(
                ui,
                "No node selected",
                "Choose a node from Inventory before configuring plugins.",
                cta,
            ) {
                self.session.selected_view = View::Nodes;
            }
            return;
        };

        let layout = layout::plugin_pane_layout(ui.available_size());
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(layout.catalog_width, layout.height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    panel(ui, "Catalog", |ui| {
                        list::render_plugin_list(self, ui, &node);
                    });
                },
            );

            ui.add_space(layout.gap);

            ui.allocate_ui_with_layout(
                egui::vec2(layout.activation_width, layout.height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    panel(ui, "Activation", |ui| {
                        details::render_plugin_details(self, ui, &node);
                    });
                },
            );
        });
    }
}
