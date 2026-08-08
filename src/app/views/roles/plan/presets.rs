use super::*;

use crate::app::domain::role_availability;

/// Height of one duty in the list. Eight of them plus the target facts and the
/// Apply Role button have to fit a panel that does not scroll.
const PRESET_HEIGHT: f32 = 26.0;

/// Narrowest a duty button may be before the pair is stacked instead. Sized for
/// the longest label, "State Validator", at the app's 13pt body font.
const PRESET_MIN_WIDTH: f32 = 112.0;

impl NeoNexusApp {
    pub(in crate::app::views::roles) fn render_role_presets(&mut self, ui: &mut egui::Ui) {
        let node_type = self.selected_node().map(|node| node.node_type);
        self.render_preset_list(ui, node_type);

        ui.add_space(theme::SM);
        hr_tight(ui);
        // One explanation, for the duty actually selected. Eight of them at once
        // pushed Apply Role ~354pt below a panel that does not scroll, so the
        // only control on this page could not be clicked — and the two duties
        // added most recently could not be reached at all.
        self.render_selected_preset_note(ui, node_type);

        ui.add_space(theme::SM);
        self.render_selected_role_target(ui);
        self.render_apply_role_button(ui);
    }

    /// The duties, two to a row when the panel is wide enough for it.
    ///
    /// Eight duties stacked one per row is 8 rows of vertical cost for a list
    /// whose labels are two words long. Pairing them halves that, and the space
    /// goes to the control this page exists for.
    ///
    /// A duty the selected node's client cannot perform is offered disabled
    /// rather than hidden, so an operator can see it exists — the reason appears
    /// below, with the selected duty's own.
    fn render_preset_list(&mut self, ui: &mut egui::Ui, node_type: Option<NodeType>) {
        let paired = fits_side_by_side_at(ui.available_width(), PRESET_MIN_WIDTH);
        let per_row = if paired { 2 } else { 1 };
        for chunk in NodeRole::ALL.chunks(per_row) {
            ui.horizontal(|ui| {
                let width = (ui.available_width() - ui.spacing().item_spacing.x) / per_row as f32;
                for role in chunk {
                    self.render_preset_button(ui, *role, node_type, width);
                }
            });
        }
    }

    fn render_preset_button(
        &mut self,
        ui: &mut egui::Ui,
        role: NodeRole,
        node_type: Option<NodeType>,
        width: f32,
    ) {
        let usable =
            node_type.is_none_or(|node_type| role_availability(node_type, role).is_supported());
        let selected = self.selected_role == role;
        if ui
            .add_enabled(
                usable,
                egui::Button::new(role.label())
                    .selected(selected)
                    .min_size(egui::vec2(width.max(PRESET_MIN_WIDTH), PRESET_HEIGHT)),
            )
            .clicked()
        {
            self.selected_role = role;
        }
    }

    /// What the selected duty does, or why this client cannot perform it.
    fn render_selected_preset_note(&self, ui: &mut egui::Ui, node_type: Option<NodeType>) {
        let role = self.selected_role;
        let availability = node_type.map(|node_type| role_availability(node_type, role));
        match availability.and_then(|state| state.reason()) {
            Some(reason) => {
                ui.label(
                    egui::RichText::new(format!("{} is unavailable here", role.label()))
                        .strong()
                        .color(status_color(NodeStatus::Error)),
                );
                ui.label(egui::RichText::new(reason).color(muted_text()));
            }
            None => {
                ui.label(egui::RichText::new(role.label()).strong());
                ui.label(egui::RichText::new(role.description()).color(muted_text()));
            }
        }
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

    /// Applying a duty is the only thing this page does, so the button and the
    /// reason it is unavailable stay together at the bottom of the surface.
    fn render_apply_role_button(&mut self, ui: &mut egui::Ui) {
        let selected = self.selected_node().cloned();
        let can_apply = selected
            .as_ref()
            .is_some_and(|node| !node.status.is_active())
            && selected.as_ref().is_some_and(|node| {
                role_availability(node.node_type, self.selected_role).is_supported()
            });
        ui.add_space(theme::SM);
        if ui
            .add_enabled(can_apply, egui::Button::new("Apply Role"))
            .clicked()
        {
            self.apply_selected_role();
        }
    }
}
