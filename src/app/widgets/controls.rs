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

/// Wraps a row of mutually-exclusive filter chips in a hairline-bordered pill so
/// single-select filters read as one macOS-style segmented control rather than a
/// loose row of toggles. The chips and their behaviour are unchanged.
pub(in crate::app) fn chip_pill(ui: &mut egui::Ui, add_chips: impl FnOnce(&mut egui::Ui)) {
    let stroke = ui.style().visuals.widgets.noninteractive.bg_stroke;
    egui::Frame::new()
        .fill(theme::panel_fill())
        .stroke(stroke)
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::symmetric(3, 2))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            ui.horizontal(add_chips);
        });
}

pub(in crate::app) fn pagination_bar(
    ui: &mut egui::Ui,
    page: &mut usize,
    total_pages: usize,
    item_count: usize,
) {
    ui.horizontal(|ui| {
        ui.label(theme::muted_body(format!("Items: {item_count}")));
        ui.separator();
        ui.label(theme::muted_body(format!(
            "Page {} / {}",
            *page + 1,
            total_pages
        )));
        chip_pill(ui, |ui| {
            if ui
                .add_enabled(*page > 0, egui::Button::new("Previous"))
                .clicked()
            {
                *page -= 1;
            }
            if ui
                .add_enabled(*page + 1 < total_pages, egui::Button::new("Next"))
                .clicked()
            {
                *page += 1;
            }
        });
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
