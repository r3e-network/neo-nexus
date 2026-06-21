use eframe::egui;

use crate::types::NodeConfig;

use super::super::super::super::{view::View, NeoNexusApp};

impl NeoNexusApp {
    pub(super) fn render_inspector_actions(&mut self, ui: &mut egui::Ui, node: &NodeConfig) {
        ui.add_space(8.0);
        self.render_node_lifecycle_actions(ui, node);
        self.render_workspace_jump_actions(ui);
    }

    fn render_node_lifecycle_actions(&mut self, ui: &mut egui::Ui, node: &NodeConfig) {
        ui.horizontal(|ui| {
            let running = node.status.is_running();
            if ui
                .add_enabled(!running, egui::Button::new("Start"))
                .clicked()
            {
                self.start_selected_node();
            }
            if ui.add_enabled(running, egui::Button::new("Stop")).clicked() {
                self.stop_selected_node();
            }
            if ui
                .add_enabled(running, egui::Button::new("Restart"))
                .clicked()
            {
                self.restart_selected_node();
            }
        });
    }

    fn render_workspace_jump_actions(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Roles").clicked() {
                self.selected_view = View::Roles;
            }
            if ui.button("Plugins").clicked() {
                self.selected_view = View::Plugins;
            }
            if ui.button("Config").clicked() {
                self.selected_view = View::Config;
            }
            if ui.button("Logs").clicked() {
                self.selected_view = View::Logs;
            }
        });
    }
}
