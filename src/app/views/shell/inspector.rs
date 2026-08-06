mod actions;
mod node;
mod runtime;
mod section;

use eframe::egui;

use super::super::super::{
    theme,
    view::View,
    widgets::{empty_state_with_action, page_chrome, status_badge},
    NeoNexusApp,
};

pub(in crate::app) use section::InspectorSection;

impl NeoNexusApp {
    /// The right-hand inspector: a selection header, a section switcher, and a
    /// stack of flat cards. The switcher exists because the panel is a fixed
    /// height with no scrolling — without it the Process, Lifecycle and Runtime
    /// blocks were painted past the panel edge and never seen.
    pub(in crate::app) fn render_inspector_panel(&mut self, ui: &mut egui::Ui) {
        let Some(node) = self.selected_node().cloned() else {
            self.render_empty_inspector(ui);
            return;
        };

        ui.horizontal(|ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
            ui.label(theme::section_title(truncated_node_name(&node.name)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                status_badge(ui, node.status);
            });
        });
        ui.add_space(theme::SM);

        let mut index = self.session.inspector_section.index();
        let labels: Vec<&str> = InspectorSection::ALL
            .iter()
            .map(|section| section.label())
            .collect();
        if page_chrome(ui, None, Some((&labels, &mut index))) {
            self.session.inspector_section = InspectorSection::from_index(index);
        }

        match self.session.inspector_section {
            InspectorSection::Overview => self.render_inspector_overview(ui, &node),
            InspectorSection::Paths => self.render_inspector_paths(ui, &node),
            InspectorSection::Process => self.render_inspector_process(ui, &node),
        }
    }

    fn render_empty_inspector(&mut self, ui: &mut egui::Ui) {
        ui.label(theme::section_title("Inspector"));
        ui.add_space(2.0);
        ui.label(theme::muted_body("Selection and runtime details"));
        ui.add_space(theme::SM);
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
    }
}

pub(super) fn truncated_node_name(name: &str) -> String {
    super::super::super::text::truncate_middle(name, 24)
}
