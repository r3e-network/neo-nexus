use eframe::egui;

use crate::app::domain::NodeConfig;

use super::super::super::super::{theme, view::View, widgets, NeoNexusApp};

impl NeoNexusApp {
    pub(super) fn render_inspector_actions(&mut self, ui: &mut egui::Ui, node: &NodeConfig) {
        ui.add_space(theme::SM);
        self.render_node_lifecycle_actions(ui, node);
        self.render_workspace_jump_actions(ui);
    }

    fn render_node_lifecycle_actions(&mut self, ui: &mut egui::Ui, node: &NodeConfig) {
        ui.horizontal(|ui| {
            let running = node.status.is_running();
            if widgets::secondary_button_enabled(ui, "Start", !running).clicked() {
                self.start_selected_node();
            }
            if widgets::secondary_button_enabled(ui, "Stop", running).clicked() {
                self.stop_selected_node();
            }
            if widgets::secondary_button_enabled(ui, "Restart", running).clicked() {
                self.restart_selected_node();
            }
        });
    }

    fn render_workspace_jump_actions(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if widgets::secondary_button(ui, "Roles").clicked() {
                self.session.selected_view = View::Roles;
            }
            if widgets::secondary_button(ui, "Plugins").clicked() {
                self.session.selected_view = View::Plugins;
            }
            if widgets::secondary_button(ui, "Config").clicked() {
                self.session.selected_view = View::Config;
            }
            if widgets::secondary_button(ui, "Logs").clicked() {
                self.session.selected_view = View::Logs;
            }
        });
    }
}
