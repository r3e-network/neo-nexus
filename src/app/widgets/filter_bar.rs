use eframe::egui;

use crate::app::theme;

/// Search field for list surfaces. Returns `true` when the query changed.
pub(in crate::app) fn filter_bar(ui: &mut egui::Ui, query: &mut String, hint: &str) -> bool {
    let response = ui.add_sized(
        [ui.available_width().max(120.0), 28.0],
        egui::TextEdit::singleline(query).hint_text(hint),
    );
    response.changed()
}

/// Single selectable chip used inside `chip_pill` filter rows.
///
/// Styled to match a segmented-control segment exactly — the selected chip is a
/// raised card-coloured pill with an accent label lifted out of the recessed
/// track, never a filled accent block. `Button::selectable` is used rather than
/// a plain `Button` so the chip keeps its selected-state accessibility
/// semantics and stays reachable by keyboard.
pub(in crate::app) fn filter_chip(ui: &mut egui::Ui, label: &str, selected: bool) -> bool {
    let text = if selected {
        theme::body(label).color(theme::accent_text()).strong()
    } else {
        theme::body(label).color(theme::muted_text())
    };
    let mut chip =
        egui::Button::selectable(selected, text).corner_radius(egui::CornerRadius::same(7));
    chip = if selected {
        chip.fill(theme::card_surface())
            .stroke(theme::hairline_strong())
    } else {
        chip.fill(egui::Color32::TRANSPARENT)
            .stroke(egui::Stroke::NONE)
    };
    ui.add(chip).clicked()
}
