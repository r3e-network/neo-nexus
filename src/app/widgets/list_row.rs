//! Shared selectable list-row chrome (v3.1 selection matrix).
//! All inventory/journal/readiness list cards should wrap content with
//! [`list_row_frame`] so selection fill/stroke stay consistent.

use eframe::egui;

use crate::app::theme::{self, card_surface};

/// Frozen selection matrix: accent×0.16 fill, radius 10, hairline when idle.
pub(in crate::app) const LIST_SELECTED_MULTIPLY: f32 = 0.16;
pub(in crate::app) const LIST_CORNER_RADIUS: u8 = 10;

/// Draw a clickable list row frame. `fixed_height` reserves a slot when the
/// page pads empty cells (inventory Comfortable 44 / Compact 40, fleet 52);
/// pass `None` for content-height rows (readiness / action queue).
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

    // Compact single-line slots (≤40) need tighter vertical padding so the
    // outer frame lands on the design height without clipping badges.
    let v_margin: i8 = match fixed_height {
        Some(h) if h <= 40.0 => 4,
        _ => 8,
    };
    let h_margin: i8 = 10;

    let response = egui::Frame::new()
        .fill(fill)
        .stroke(stroke)
        .corner_radius(egui::CornerRadius::same(LIST_CORNER_RADIUS))
        .inner_margin(egui::Margin::symmetric(h_margin, v_margin))
        .show(ui, |ui| {
            if let Some(height) = fixed_height {
                let content_h = (height - f32::from(v_margin) * 2.0).max(1.0);
                ui.set_min_size(egui::vec2(width - 4.0, content_h));
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
