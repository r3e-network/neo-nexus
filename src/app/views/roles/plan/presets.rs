use super::*;

const ROLE_ROWS: usize = 5;

impl NeoNexusApp {
    pub(in crate::app::views::roles) fn render_role_presets(&mut self, ui: &mut egui::Ui) {
        for row in 0..ROLE_ROWS {
            if let Some(role) = NodeRole::ALL.get(row).copied() {
                let selected = self.selected_role == role;
                if ui
                    .add_sized(
                        [ui.available_width(), 28.0],
                        egui::Button::new(role.label()).selected(selected),
                    )
                    .clicked()
                {
                    self.selected_role = role;
                }
                ui.label(egui::RichText::new(role.description()).color(muted_text()));
            } else {
                ui.label(" ");
            }
            ui.add_space(4.0);
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
                        egui::RichText::new(node.status.to_string())
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
            .is_some_and(|node| !matches!(node.status, NodeStatus::Running | NodeStatus::Starting));
        ui.add_space(8.0);
        if ui
            .add_enabled(can_apply, egui::Button::new("Apply Role"))
            .clicked()
        {
            self.apply_selected_role();
        }
    }
}
