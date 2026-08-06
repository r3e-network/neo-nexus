//! Recessed detail surface nested inside a card.
//!
//! Six pages hand-rolled a byte-identical `Frame` for this (process detail,
//! event journal, action queue, readiness checks, matrix table, overview
//! actions). They are one component: a track-toned, hairline-bordered inset
//! that reads as *inside* its parent card rather than floating on it.

use eframe::egui;

use crate::app::theme;

use super::layout::CARD_CORNER_RADIUS;

/// A recessed panel for detail content nested inside a card.
pub(in crate::app) fn inset_card(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(theme::track_surface())
        .stroke(theme::hairline())
        .corner_radius(egui::CornerRadius::same(CARD_CORNER_RADIUS))
        .inner_margin(egui::Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            add_contents(ui);
        });
}
