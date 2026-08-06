//! Hairline rules.
//!
//! `ui.separator()` auto-orients from the surrounding layout and allocates a
//! full `interact_size` of space, which is why the status bar overflows its
//! fixed 28pt panel. These two helpers are explicit about direction and about
//! how much room they take, so a rule can be dropped into a fixed-height row
//! without pushing content out of the panel.

use eframe::egui;

/// Horizontal rule across the available width, with symmetric breathing room.
pub(in crate::app) fn hr(ui: &mut egui::Ui) {
    ui.add_space(crate::app::theme::SM);
    rule(ui, egui::vec2(ui.available_width(), 1.0), true);
    ui.add_space(crate::app::theme::SM);
}

/// Horizontal rule with the tight breathing room a dense grid wants. A full
/// [`hr`] between every row of a metric grid spends more height on rules than
/// on data.
pub(in crate::app) fn hr_tight(ui: &mut egui::Ui) {
    ui.add_space(crate::app::theme::XS);
    rule(ui, egui::vec2(ui.available_width(), 1.0), true);
    ui.add_space(crate::app::theme::XS);
}

/// Vertical rule of an exact height, for divider-separated inline pairs (the
/// status bar). Takes only its own width, so it never inflates the row.
pub(in crate::app) fn vr(ui: &mut egui::Ui, height: f32) {
    rule(ui, egui::vec2(1.0, height), false);
}

fn rule(ui: &mut egui::Ui, size: egui::Vec2, horizontal: bool) {
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }
    let color = crate::app::theme::hairline().color;
    let line = if horizontal {
        egui::Rect::from_min_size(
            egui::pos2(rect.min.x, rect.center().y),
            egui::vec2(rect.width(), 1.0),
        )
    } else {
        egui::Rect::from_min_size(
            egui::pos2(rect.center().x, rect.min.y),
            egui::vec2(1.0, rect.height()),
        )
    };
    ui.painter().rect_filled(line, 0, color);
}
