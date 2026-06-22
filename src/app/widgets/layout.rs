use eframe::egui;

use crate::app::theme::{muted_text, panel_fill};

pub(in crate::app) fn metric_tile(ui: &mut egui::Ui, title: &str, value: &str, caption: &str) {
    let width = ((ui.available_width() - 24.0) / 4.0).max(150.0);
    ui.group(|ui| {
        ui.set_min_size(egui::vec2(width, 84.0));
        ui.label(egui::RichText::new(title).color(muted_text()));
        ui.label(egui::RichText::new(value).size(24.0).strong());
        ui.label(caption);
    });
}

pub(in crate::app) fn panel(
    ui: &mut egui::Ui,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    egui::Frame::group(ui.style())
        .fill(panel_fill())
        .show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            ui.horizontal(|ui| {
                ui.strong(title);
            });
            ui.separator();
            add_contents(ui);
        });
}

pub(in crate::app) fn empty_state(ui: &mut egui::Ui, title: &str, message: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() * 0.35);
        ui.heading(title);
        ui.label(message);
    });
}
