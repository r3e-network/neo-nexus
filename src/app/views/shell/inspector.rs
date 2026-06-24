mod actions;
mod node;
mod runtime;

use eframe::egui;

use super::super::super::{text::truncate_middle, theme, widgets::empty_state, NeoNexusApp};

impl NeoNexusApp {
    pub(in crate::app) fn render_inspector_panel(&mut self, ui: &mut egui::Ui) {
        render_inspector_header(ui);

        let Some(node) = self.selected_node().cloned() else {
            empty_state(ui, "Nothing selected", "Select a node from Inventory.");
            ui.separator();
            self.render_runtime_facts(ui);
            return;
        };

        self.render_selected_node_inspector(ui, &node);
        ui.separator();
        self.render_runtime_facts(ui);
    }
}

fn render_inspector_header(ui: &mut egui::Ui) {
    ui.add_space(theme::MD);
    ui.horizontal(|ui| {
        ui.add_space(theme::MD);
        ui.vertical(|ui| {
            ui.label(theme::section_title("Inspector"));
            ui.label(theme::muted_body("Selection and runtime details"));
        });
    });
    ui.separator();
}

pub(super) fn truncated_node_name(name: &str) -> String {
    truncate_middle(name, 28)
}
