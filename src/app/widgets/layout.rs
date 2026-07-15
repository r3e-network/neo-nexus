use eframe::egui;

use crate::app::theme::{self, card_shadow, card_surface};

/// Shared surface for cards and panels: a filled, softly rounded container with
/// a hairline border and a faint drop shadow so content lifts cleanly off the
/// workspace background and reads as an elevated macOS card rather than a flat
/// egui rectangle.
#[allow(dead_code)] // used by form kit; re-export kept for shared card chrome
pub(in crate::app) fn card_frame(style: &egui::Style) -> egui::Frame {
    egui::Frame::new()
        .fill(card_surface())
        .stroke(style.visuals.widgets.noninteractive.bg_stroke)
        .corner_radius(egui::CornerRadius::same(12))
        .shadow(card_shadow())
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
            ui.add_space(theme::XS);
            ui.label(theme::metric_value(value));
            ui.add_space(theme::XS);
            ui.label(theme::muted_body(caption));
        });
    });
}

/// Compact labelled stat for dense side panels: a caption over a value, with no
/// card chrome (the surrounding panel provides the surface). Used where a full
/// metric tile would be too heavy but a plain fact row lacks presence — e.g. the
/// inventory's Total/Running/Stopped/Visible counts.
pub(in crate::app) fn mini_stat(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.vertical(|ui| {
        ui.label(theme::label_caption(label));
        ui.label(theme::section_title(value).strong());
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

/// Header row for an `egui::Grid` data table: renders each column label with the
/// shared macOS-style column-header treatment and closes the row.
pub(in crate::app) fn grid_header(ui: &mut egui::Ui, headers: &[&str]) {
    for header in headers {
        ui.label(theme::column_header(*header));
    }
    ui.end_row();
}

pub(in crate::app) fn empty_state(ui: &mut egui::Ui, title: &str, message: &str) {
    empty_state_with_action(ui, title, message, None);
}

/// Empty state with an optional primary CTA. Returns `true` when the CTA is
/// clicked so callers can route to create/import flows.
pub(in crate::app) fn empty_state_with_action(
    ui: &mut egui::Ui,
    title: &str,
    message: &str,
    action: Option<&str>,
) -> bool {
    let mut clicked = false;
    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() * 0.28);
        // A muted tray pictogram anchors the empty state with a focal mark, the
        // way a macOS empty list does, so the guidance reads as intentional
        // rather than as bare placeholder words. Rendered at the metric-value
        // size (24pt, on the type scale) and the muted tone so it stays a quiet
        // presence above the guidance, not an alarming one.
        ui.label(theme::metric_value(theme::empty_glyph()).color(theme::muted_text()));
        ui.add_space(theme::SM);
        ui.label(theme::page_title(title));
        ui.add_space(theme::XS);
        ui.label(theme::muted_body(message));
        if let Some(label) = action {
            ui.add_space(theme::MD);
            if super::controls::primary_button(ui, label).clicked() {
                clicked = true;
            }
        }
    });
    clicked
}
