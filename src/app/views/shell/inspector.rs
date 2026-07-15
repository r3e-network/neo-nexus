mod actions;
mod node;
mod runtime;

use eframe::egui;

use super::super::super::{
    theme,
    view::View,
    widgets::{empty_state_with_action, status_badge},
    NeoNexusApp,
};

impl NeoNexusApp {
    pub(in crate::app) fn render_inspector_panel(&mut self, ui: &mut egui::Ui) {
        render_inspector_header(ui);

        let Some(node) = self.selected_node().cloned() else {
            if empty_state_with_action(
                ui,
                "Nothing selected",
                "Select a node from Inventory to inspect lifecycle, ports, and process facts.",
                if self.fleet.nodes.is_empty() {
                    Some("Create node")
                } else {
                    None
                },
            ) {
                self.session.selected_view = View::Nodes;
            }
            ui.add_space(theme::MD);
            ui.separator();
            ui.add_space(theme::SM);
            self.render_runtime_facts(ui);
            return;
        };

        ui.horizontal(|ui| {
            ui.label(theme::section_title(truncated_node_name(&node.name)));
            ui.add_space(theme::SM);
            status_badge(ui, node.status);
        });
        ui.add_space(theme::SM);
        ui.separator();
        ui.add_space(theme::SM);

        self.render_selected_node_inspector(ui, &node);
        ui.add_space(theme::MD);
        ui.separator();
        ui.add_space(theme::SM);
        self.render_runtime_facts(ui);
    }
}

fn render_inspector_header(ui: &mut egui::Ui) {
    ui.add_space(theme::SM);
    ui.vertical(|ui| {
        ui.label(theme::section_title("Inspector"));
        ui.add_space(2.0);
        ui.label(theme::muted_body("Selection and runtime details"));
    });
    ui.add_space(theme::SM);
    ui.separator();
    ui.add_space(theme::SM);
}

pub(super) fn truncated_node_name(name: &str) -> String {
    super::super::super::text::truncate_middle(name, 28)
}
