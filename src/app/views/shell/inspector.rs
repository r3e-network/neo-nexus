mod actions;
mod node;
mod runtime;

use eframe::egui;

use super::super::super::{
    text::truncate_middle, theme::muted_text, widgets::empty_state, NeoNexusApp,
};

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
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        ui.add_space(10.0);
        ui.vertical(|ui| {
            ui.heading("Inspector");
            ui.label(egui::RichText::new("Selection and runtime details").color(muted_text()));
        });
    });
    ui.separator();
}

pub(super) fn truncated_node_name(name: &str) -> String {
    truncate_middle(name, 28)
}
