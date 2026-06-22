use eframe::egui;

use crate::app::theme;

/// macOS-style "default button": the primary, accent-filled confirm action of a
/// form. Use for the single dominant action (Save / Create / Import); leave
/// secondary actions as plain bordered buttons.
pub(in crate::app) fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(text).color(theme::on_accent()))
            .fill(theme::accent())
            .min_size(egui::vec2(96.0, 28.0)),
    )
}

pub(in crate::app) fn pagination_bar(
    ui: &mut egui::Ui,
    page: &mut usize,
    total_pages: usize,
    item_count: usize,
) {
    ui.horizontal(|ui| {
        ui.label(format!("Items: {item_count}"));
        ui.separator();
        if ui
            .add_enabled(*page > 0, egui::Button::new("Previous"))
            .clicked()
        {
            *page -= 1;
        }
        ui.label(format!("Page {} / {}", *page + 1, total_pages));
        if ui
            .add_enabled(*page + 1 < total_pages, egui::Button::new("Next"))
            .clicked()
        {
            *page += 1;
        }
    });
}

pub(in crate::app) fn labeled_text(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add_sized(
            [ui.available_width().max(120.0), 24.0],
            egui::TextEdit::singleline(value),
        );
    });
}

pub(in crate::app) fn labeled_combo(
    ui: &mut egui::Ui,
    label: &str,
    id: &'static str,
    selected: String,
    add_items: impl FnOnce(&mut egui::Ui),
) {
    ui.horizontal(|ui| {
        ui.label(label);
        egui::ComboBox::from_id_salt(id)
            .selected_text(selected)
            .width(ui.available_width().max(120.0))
            .show_ui(ui, add_items);
    });
}
