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
pub(in crate::app) fn filter_chip(ui: &mut egui::Ui, label: &str, selected: bool) -> bool {
    ui.selectable_label(selected, theme::body(label)).clicked()
}
