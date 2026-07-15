mod plan;
mod private_network;
mod section;

use eframe::egui;

use crate::app::domain::{PrivateNetworkPlanner, RolePlanner};

use super::super::{
    theme,
    widgets::{metric_row, panel, segmented_control},
    NeoNexusApp,
};

pub(in crate::app) use section::RolesSection;

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

        let changes = role_plan
            .as_ref()
            .map_or_else(|| "-".to_string(), |plan| plan.change_count().to_string());
        let private_plan_count = private_plan.nodes.len().to_string();
        let runtime_label = selected_node
            .as_ref()
            .map_or(self.private_network_runtime, |node| node.node_type)
            .to_string();
        metric_row(
            ui,
            &[
                ("Role", self.selected_role.label(), "selected preset"),
                ("Changes", &changes, "plugin states"),
                ("Private Plan", &private_plan_count, "planned nodes"),
                ("Runtime", &runtime_label, "selected"),
            ],
        );

        ui.add_space(theme::MD);
        let mut index = self.sections.roles as usize;
        let labels = RolesSection::ALL.map(RolesSection::label);
        if segmented_control(ui, &labels, &mut index) {
            self.sections.roles = RolesSection::ALL[index];
        }
        ui.add_space(theme::MD);

        match self.sections.roles {
            RolesSection::Presets => panel(ui, "Role presets", |ui| {
                self.render_role_presets(ui);
            }),
            RolesSection::Plan => panel(ui, "Selected role plan", |ui| {
                self.render_selected_role_plan(ui);
            }),
            RolesSection::PrivateNetwork => panel(ui, "Private network planner", |ui| {
                self.render_private_network_plan(ui);
            }),
        }
    }
}
