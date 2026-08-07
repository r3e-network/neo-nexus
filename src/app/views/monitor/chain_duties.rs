//! Whether the chain agrees this node can perform its duty.
//!
//! A node can be configured for the Oracle role, running, and answering RPC,
//! and still do nothing at all — because the committee has not designated its
//! key. That fact is not in any log or config file, so the only honest place to
//! surface it is next to the node's health.
//!
//! There is no Designate button and there will not be one: designation is a
//! committee-witnessed transaction signed by keys NeoNexus does not hold. What
//! the panel can do is state the position and name who can change it.

use eframe::egui;

use crate::app::{
    domain::{NodeConfig, NodeRole},
    theme,
    widgets::{callout, empty_state, metric_grid, secondary_button, CalloutKind},
    NeoNexusApp,
};

impl NeoNexusApp {
    pub(in crate::app::views::monitor) fn render_chain_duties(&mut self, ui: &mut egui::Ui) {
        let Some(node) = self.selected_node().cloned() else {
            empty_state(
                ui,
                "No node selected",
                "Select a node to see whether the chain has granted it the duty it is configured for.",
            );
            return;
        };
        let role = self.repository.load_node_role(&node.id).unwrap_or_default();
        let Some(role) = role else {
            empty_state(
                ui,
                "No duty assigned",
                "Apply a role from Nodes › Roles; chain duties only apply once a node has one.",
            );
            return;
        };

        self.render_duty_summary(ui, &node, role);
        ui.add_space(theme::SM);
        self.render_designation_state(ui, &node, role);
    }

    fn render_duty_summary(&self, ui: &mut egui::Ui, node: &NodeConfig, role: NodeRole) {
        let designation = role.designation().map_or_else(
            || "none required".to_string(),
            |role| role.label().to_string(),
        );
        metric_grid(
            ui,
            &[
                ("Node", node.name.clone()),
                ("Duty", role.to_string()),
                ("Runtime", node.node_type.to_string()),
                ("Designation", designation),
            ],
        );
    }

    fn render_designation_state(&mut self, ui: &mut egui::Ui, node: &NodeConfig, role: NodeRole) {
        let Some(chain_role) = role.designation() else {
            ui.label(theme::muted_body(format!(
                "The {role} duty is granted by configuration alone. A validator is elected by NEO \
                 holders through the committee vote, not designated by it."
            )));
            return;
        };

        match self.chain.designation_for(&node.id) {
            Some(Ok(designation)) => {
                let kind = if designation.is_designated() {
                    CalloutKind::Success
                } else {
                    CalloutKind::Warning
                };
                callout(ui, kind, chain_role.label(), &designation.summary());
            }
            Some(Err(error)) => callout(
                ui,
                CalloutKind::Danger,
                "Could not read the chain",
                error.message(),
            ),
            None => {
                ui.label(theme::muted_body(
                    "Designation has not been read for this node yet.",
                ));
            }
        }

        ui.add_space(theme::SM);
        if secondary_button(ui, "Check designation")
            .on_hover_text("Ask this node who currently holds the role")
            .clicked()
        {
            self.check_selected_designation();
        }
        ui.add_space(theme::XS);
        ui.label(theme::caption(
            "Read-only. Designating a key is a committee-witnessed transaction; NeoNexus holds no \
             keys and cannot sign one.",
        ));
    }
}
