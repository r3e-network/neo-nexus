use eframe::egui;

use crate::app::theme::{self, panel_fill};

/// Shared surface for cards and panels: a filled, softly rounded container with
/// a hairline border so content lifts cleanly off the workspace background.
fn card_frame(style: &egui::Style) -> egui::Frame {
    egui::Frame::new()
        .fill(panel_fill())
        .stroke(style.visuals.widgets.noninteractive.bg_stroke)
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(14, 12))
}

/// Render a row of equal-width metric cards. Columns share the available width
/// evenly, so a four-up summary stays aligned regardless of value lengths.
pub(in crate::app) fn metric_row(ui: &mut egui::Ui, tiles: &[(&str, &str, &str)]) {
    if tiles.is_empty() {
        return;
    }
    ui.columns(tiles.len(), |columns| {
        for (column, &(title, value, caption)) in columns.iter_mut().zip(tiles) {
            metric_tile(column, title, value, caption);
        }
    });
}

fn metric_tile(ui: &mut egui::Ui, title: &str, value: &str, caption: &str) {
    card_frame(ui.style()).show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.vertical(|ui| {
            ui.label(theme::label_caption(title));
            ui.add_space(2.0);
            ui.label(theme::metric_value(value));
            ui.label(theme::muted_body(caption));
        });
    });
}

pub(in crate::app) fn panel(
    ui: &mut egui::Ui,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    card_frame(ui.style()).show(ui, |ui| {
        ui.set_min_size(ui.available_size());
        ui.label(theme::section_title(title));
        ui.add_space(theme::SM);
        ui.separator();
        ui.add_space(theme::XS);
        add_contents(ui);
    });
}

pub(in crate::app) fn empty_state(ui: &mut egui::Ui, title: &str, message: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() * 0.35);
        ui.label(theme::page_title(title));
        ui.add_space(theme::XS);
        ui.label(theme::muted_body(message));
    });
}
