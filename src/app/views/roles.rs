mod plan;
mod private_network;

use eframe::egui;

use crate::app::domain::{PrivateNetworkPlanner, RolePlanner};

use super::super::{
    widgets::{metric_tile, panel},
    NeoNexusApp,
};

const PANEL_GAP: f32 = 8.0;

impl NeoNexusApp {
    pub(super) fn render_roles(&mut self, ui: &mut egui::Ui) {
        let selected_node = self.selected_node().cloned();
        let role_plan = selected_node
            .as_ref()
            .map(|node| RolePlanner::plan(node, self.selected_role));
        let private_plan = PrivateNetworkPlanner::plan(
            self.private_network_template,
            self.private_network_runtime,
        );

        ui.horizontal(|ui| {
            metric_tile(ui, "Role", self.selected_role.label(), "selected preset");
            metric_tile(
                ui,
                "Changes",
                &role_plan
                    .as_ref()
                    .map_or_else(|| "-".to_string(), |plan| plan.change_count().to_string()),
                "plugin states",
            );
            metric_tile(
                ui,
                "Private Plan",
                &private_plan.nodes.len().to_string(),
                "planned nodes",
            );
            metric_tile(
                ui,
                "Runtime",
                &selected_node
                    .as_ref()
                    .map_or(self.private_network_runtime, |node| node.node_type)
                    .to_string(),
                "selected",
            );
        });

        ui.add_space(10.0);
        let available = ui.available_size();
        let top_height = (available.y * 0.52).clamp(260.0, 340.0);
        let left_width = (available.x * 0.38).clamp(320.0, 460.0);

        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(left_width, top_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    panel(ui, "Role presets", |ui| {
                        self.render_role_presets(ui);
                    });
                },
            );

            ui.add_space(PANEL_GAP);

            ui.allocate_ui_with_layout(
                egui::vec2(
                    (available.x - left_width - PANEL_GAP).max(400.0),
                    top_height,
                ),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    panel(ui, "Selected role plan", |ui| {
                        self.render_selected_role_plan(ui);
                    });
                },
            );
        });

        ui.add_space(PANEL_GAP);
        let bottom = ui.available_size();
        ui.allocate_ui_with_layout(
            egui::vec2(bottom.x, bottom.y),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                panel(ui, "Private network planner", |ui| {
                    self.render_private_network_plan(ui);
                });
            },
        );
    }
}
