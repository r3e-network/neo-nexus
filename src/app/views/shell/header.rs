mod actions;
mod menu;
mod state;

use eframe::egui;

use self::{actions::render_node_action_buttons, menu::render_application_menu};
use super::super::super::{theme, widgets::vr, NeoNexusApp};

/// Height of the rule between the node actions and the application menus.
/// `ui.separator()` would claim a full `interact_size` of vertical room, which
/// this bar does not have.
const HEADER_DIVIDER_HEIGHT: f32 = 20.0;

// Re-exported for the menu submodule's `use super::View`.
pub(super) use super::super::super::view::View;

impl NeoNexusApp {
    pub(in crate::app) fn render_application_header(&mut self, ui: &mut egui::Ui) {
        // The title/subtitle stack plus the default item spacing came to 47pt
        // inside a bar whose content box is 40pt, so the header frame painted
        // 7pt below its own panel in every view. The stack is tightened rather
        // than the bar grown: `CHROME_HEADER_HEIGHT` is a density-invariant
        // contract three test files pin.
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.add_space(theme::XS);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.label(theme::page_title(self.session.selected_view.title()));
                ui.label(theme::muted_body(self.session.selected_view.subtitle()));
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(theme::SM);
                render_node_action_buttons(self, ui);
                ui.add_space(theme::SM);
                vr(ui, HEADER_DIVIDER_HEIGHT);
                ui.add_space(theme::SM);
                render_application_menu(self, ui);
            });
        });
    }
}
