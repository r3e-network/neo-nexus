mod checks;
mod filters;

use eframe::egui;

use crate::diagnostics::FleetDiagnostics;

use super::{
    super::super::{
        text::truncate_middle,
        view::View,
        widgets::{empty_state, fact},
        NeoNexusApp,
    },
    helpers::score_color,
};
use checks::render_checks;
use filters::render_check_filters;

impl NeoNexusApp {
    pub(super) fn render_selected_readiness(
        &mut self,
        ui: &mut egui::Ui,
        diagnostics: &FleetDiagnostics,
    ) {
        let Some(selected_id) = self.selected_node.as_deref() else {
            empty_state(ui, "No selection", "Select a node from Inventory.");
            return;
        };
        let Some(node) = diagnostics
            .nodes
            .iter()
            .find(|diagnostic| diagnostic.node_id == selected_id)
            .cloned()
        else {
            empty_state(ui, "No diagnostics", "Reload the workspace.");
            return;
        };

        ui.horizontal(|ui| {
            ui.heading(truncate_middle(&node.node_name, 28));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!("{}%", node.score))
                        .strong()
                        .color(score_color(node.score)),
                );
            });
        });
        fact(ui, "Critical", &node.critical_count().to_string());
        fact(ui, "Warnings", &node.warning_count().to_string());
        ui.separator();

        render_check_filters(self, ui, &node);
        self.clamp_readiness_check_page(&node);
        let checks = self.filtered_readiness_checks(&node);
        render_checks(self, ui, &checks);

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if ui.button("Node Studio").clicked() {
                self.selected_view = View::Nodes;
            }
            if ui.button("Plugins").clicked() {
                self.selected_view = View::Plugins;
            }
        });
    }
}
