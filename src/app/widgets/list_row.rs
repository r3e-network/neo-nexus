//! Shared selectable list-row chrome (v3.1 selection matrix).
//! All inventory/journal/readiness list cards should wrap content with
//! [`list_row_frame`] so selection fill/stroke stay consistent.

use eframe::egui;

use crate::app::theme::{self, card_surface};

/// Frozen selection matrix: accent×0.16 fill, radius 10, hairline when idle.
pub(in crate::app) const LIST_SELECTED_MULTIPLY: f32 = 0.16;
pub(in crate::app) const LIST_CORNER_RADIUS: u8 = 10;

/// Draw a clickable list row frame. `fixed_height` reserves a slot when the
/// page pads empty cells (inventory 44/56, journal 52); pass `None` for
/// content-height rows (readiness / action queue).
///
/// Returns whether the row was clicked this frame.
pub(in crate::app) fn list_row_frame(
    ui: &mut egui::Ui,
    selected: bool,
    fixed_height: Option<f32>,
    add_contents: impl FnOnce(&mut egui::Ui),
) -> bool {
    let width = ui.available_width();
    let fill = if selected {
        theme::accent().gamma_multiply(LIST_SELECTED_MULTIPLY)
    } else {
        card_surface()
    };
    let stroke = if selected {
        egui::Stroke::new(1.0, theme::accent())
    } else {
        theme::hairline()
    };

    let response = egui::Frame::new()
        .fill(fill)
        .stroke(stroke)
        .corner_radius(egui::CornerRadius::same(LIST_CORNER_RADIUS))
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            if let Some(height) = fixed_height {
                ui.set_min_size(egui::vec2(width - 4.0, (height - 8.0).max(1.0)));
            } else {
                ui.set_min_width(width - 4.0);
            }
            add_contents(ui);
        })
        .response
        .interact(egui::Sense::click());

    if response.hovered() && !selected {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response.clicked()
}
