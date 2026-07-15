//! Lightweight in-panel busy indicators for async_bus / install work.

use eframe::egui;

use crate::app::theme;

/// Compact single-line busy hint (e.g. "Checking RPC…").
pub(in crate::app) fn busy_inline(ui: &mut egui::Ui, label: &str) {
    ui.horizontal(|ui| {
        ui.spinner();
        ui.add_space(theme::XS);
        ui.label(theme::muted_body(label));
    });
}

/// Stronger busy strip for multi-step work (install/download).
pub(in crate::app) fn loading_callout(ui: &mut egui::Ui, title: &str, detail: &str) {
    egui::Frame::new()
        .fill(theme::card_surface())
        .stroke(theme::hairline())
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.add_space(theme::SM);
                ui.vertical(|ui| {
                    ui.label(theme::body(title).strong());
                    if !detail.is_empty() {
                        ui.label(theme::muted_body(detail));
                    }
                });
            });
        });
}
