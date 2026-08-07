use super::*;

use crate::app::domain::role_availability;

impl NeoNexusApp {
    pub(in crate::app::views::roles) fn render_role_presets(&mut self, ui: &mut egui::Ui) {
        // A duty the selected node's client cannot perform is offered greyed
        // out with the reason, rather than hidden or — worse — accepted and
        // then silently never performed.
        let node_type = self.selected_node().map(|node| node.node_type);
        for role in NodeRole::ALL {
            let availability = node_type.map(|node_type| role_availability(node_type, role));
            let usable = availability.is_none_or(|state| state.is_supported());
            let selected = self.selected_role == role;
            if ui
                .add_enabled(
                    usable,
                    egui::Button::new(role.label())
                        .selected(selected)
                        .min_size(egui::vec2(ui.available_width(), 28.0)),
                )
                .clicked()
            {
                self.selected_role = role;
            }
            let note = availability
                .and_then(|state| state.reason())
                .unwrap_or_else(|| role.description());
            ui.label(egui::RichText::new(note).color(muted_text()));
            ui.add_space(theme::XS);
        }

        ui.separator();
        self.render_selected_role_target(ui);
        self.render_apply_role_button(ui);
    }

    fn render_selected_role_target(&self, ui: &mut egui::Ui) {
        if let Some(node) = self.selected_node().cloned() {
            fact(ui, "Target", &truncate_middle(&node.name, 34));
            fact(ui, "Runtime", &node.node_type.to_string());
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Status").color(muted_text()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(node.status.label())
                            .strong()
                            .color(status_color(node.status)),
                    );
                });
            });
        } else {
            fact(ui, "Target", "-");
            fact(ui, "Runtime", "-");
            fact(ui, "Status", "-");
        }
    }

    fn render_apply_role_button(&mut self, ui: &mut egui::Ui) {
        let can_apply = self
            .selected_node()
            .is_some_and(|node| !node.status.is_active());
        ui.add_space(theme::SM);
        if ui
            .add_enabled(can_apply, egui::Button::new("Apply Role"))
            .clicked()
        {
            self.apply_selected_role();
        }
    }
}
