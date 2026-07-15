mod context;
mod diagnosis;
mod layout;
mod output;

use eframe::egui;

use crate::app::domain::LogReader;

use super::super::{
    view::View,
    widgets::{empty_state_with_action, panel},
    NeoNexusApp, LOG_MAX_BYTES,
};

impl NeoNexusApp {
    pub(super) fn render_logs(&mut self, ui: &mut egui::Ui) {
        let Some(node) = self.selected_node().cloned() else {
            let cta = if self.fleet.nodes.is_empty() {
                Some("Create node")
            } else {
                None
            };
            if empty_state_with_action(
                ui,
                "No node selected",
                "Choose a node from Inventory to inspect runtime logs.",
                cta,
            ) {
                self.session.selected_view = View::Nodes;
            }
            return;
        };

        let path = self.node_log_path(&node);
        let snapshot = LogReader::snapshot(&path, LOG_MAX_BYTES);
        let layout = layout::log_layout(ui.available_size());

        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(layout.context_width, layout.height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    panel(ui, "Log context", |ui| {
                        context::render_log_context(self, ui, &node, &path, &snapshot);
                    });
                },
            );

            ui.add_space(layout.gap);

            ui.allocate_ui_with_layout(
                egui::vec2(layout.output_width, layout.height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    panel(ui, "Log output", |ui| {
                        output::render_log_output(self, ui, &snapshot);
                    });
                },
            );
        });
    }
}
