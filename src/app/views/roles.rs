mod plan;
mod private_network;
mod section;

use eframe::egui;

use crate::app::domain::PrivateNetworkPlanner;

use super::super::{
    theme,
    widgets::{metric_row, page_chrome, panel},
    NeoNexusApp,
};

pub(in crate::app) use private_network::PrivateNetworkSection;
pub(in crate::app) use section::RolesSection;

impl NeoNexusApp {
    /// The Roles tab of the Nodes workspace.
    ///
    /// It carries no metric row. It used to, from when this was a top-level
    /// page: Role and Runtime restated the button the operator had just pressed
    /// and the inspector beside it, and "Private Plan" counted nodes for the
    /// private-network planner, which moved to the Network hub. Rendered inside
    /// the Nodes workspace those four metrics sat under a second header and a
    /// second tab bar, and the ~90pt they took is why Apply Role — the only
    /// control on the page — was laid out below a panel that does not scroll.
    pub(super) fn render_roles(&mut self, ui: &mut egui::Ui) {
        let mut index = self.sections.roles as usize;
        let labels = RolesSection::ALL.map(RolesSection::label);
        if page_chrome(ui, None, Some((&labels, &mut index))) {
            self.sections.roles = RolesSection::ALL[index];
        }

        match self.sections.roles {
            RolesSection::Presets => panel(ui, "Role presets", |ui| {
                self.render_role_presets(ui);
            }),
            RolesSection::Plan => panel(ui, "Selected role plan", |ui| {
                self.render_selected_role_plan(ui);
            }),
        }
    }

    /// Private-network planning, reached from the Network hub. A topology is
    /// not a property of the selected node, so it does not belong beside the
    /// role presets it used to share a page with.
    pub(super) fn render_private_network(&mut self, ui: &mut egui::Ui) {
        let private_plan = PrivateNetworkPlanner::plan(
            self.private_network_template,
            self.private_network_runtime,
        );
        metric_row(
            ui,
            &[
                (
                    "Template",
                    self.private_network_template.label(),
                    "selected topology",
                ),
                (
                    "Planned",
                    &private_plan.nodes.len().to_string(),
                    "nodes in plan",
                ),
                (
                    "Runtime",
                    &self.private_network_runtime.to_string(),
                    "committee binary",
                ),
            ],
        );
        ui.add_space(theme::MD);
        let mut index = self.sections.private_network as usize;
        let labels = PrivateNetworkSection::ALL.map(PrivateNetworkSection::label);
        if page_chrome(ui, None, Some((&labels, &mut index))) {
            self.sections.private_network = PrivateNetworkSection::ALL[index];
        }
        panel(ui, self.sections.private_network.label(), |ui| {
            self.render_private_network_plan(ui);
        });
    }
}
