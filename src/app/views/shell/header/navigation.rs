use eframe::egui;

use super::{actions::render_node_action_buttons, View};
use crate::app::NeoNexusApp;

const RESERVED_ACTION_WIDTH: f32 = 438.0;
const MIN_VIEW_BUTTON_WIDTH: f32 = 58.0;
const MAX_VIEW_BUTTON_WIDTH: f32 = 76.0;
const COMPACT_LABEL_THRESHOLD: f32 = 66.0;

impl NeoNexusApp {
    pub(super) fn render_application_navigation_row(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            let view_button_width = view_button_width(ui.available_width());
            for view in View::ALL {
                self.render_view_button(ui, view, view_button_width);
            }

            ui.separator();
            render_node_action_buttons(self, ui);
        });
    }

    fn render_view_button(&mut self, ui: &mut egui::Ui, view: View, width: f32) {
        let selected = self.selected_view == view;
        let label = if width < COMPACT_LABEL_THRESHOLD {
            view.short_label()
        } else {
            view.label()
        };

        if ui
            .add_sized([width, 30.0], egui::Button::new(label).selected(selected))
            .on_hover_text(format!("{} - {}", view.title(), view.subtitle()))
            .clicked()
        {
            self.selected_view = view;
        }
    }
}

fn view_button_width(available_width: f32) -> f32 {
    ((available_width - RESERVED_ACTION_WIDTH) / View::ALL.len() as f32)
        .clamp(MIN_VIEW_BUTTON_WIDTH, MAX_VIEW_BUTTON_WIDTH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_header_tabs_fit_minimum_window_budget() {
        let width = view_button_width(1280.0);
        let total = width * View::ALL.len() as f32 + RESERVED_ACTION_WIDTH;

        assert!(width >= MIN_VIEW_BUTTON_WIDTH);
        assert!(total <= 1280.0);
    }
}
