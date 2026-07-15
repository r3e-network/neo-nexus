//! Vertical form primitives for the v3 workbench (Node Studio and later forms).

use eframe::egui;

use crate::app::theme::{self, card_surface};

use super::layout::card_frame;

/// Label-above text field (form rows that need more breathing room than the
/// compact horizontal `labeled_text` helper).
pub(in crate::app) fn field_text(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.vertical(|ui| {
        ui.label(theme::label_caption(label));
        ui.add_space(theme::XS);
        ui.add_sized(
            [ui.available_width().max(120.0), 28.0],
            egui::TextEdit::singleline(value),
        );
    });
}

/// Label-above combo shell. Caller fills the dropdown via `add_items`.
pub(in crate::app) fn field_combo(
    ui: &mut egui::Ui,
    label: &str,
    id: &'static str,
    selected: String,
    add_items: impl FnOnce(&mut egui::Ui),
) {
    ui.vertical(|ui| {
        ui.label(theme::label_caption(label));
        ui.add_space(theme::XS);
        egui::ComboBox::from_id_salt(id)
            .selected_text(selected)
            .width(ui.available_width().max(120.0))
            .show_ui(ui, add_items);
    });
}

/// Card section with a title and optional footer action area.
#[allow(dead_code)]
pub(in crate::app) fn form_section(
    ui: &mut egui::Ui,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    card_frame(ui.style()).show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.label(theme::section_title(title));
        ui.add_space(theme::SM);
        ui.separator();
        ui.add_space(theme::SM);
        add_contents(ui);
    });
}

/// Soft inset surface for nested form groups without full card chrome.
pub(in crate::app) fn form_group(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    ui.label(theme::label_caption(title));
    ui.add_space(theme::XS);
    egui::Frame::new()
        .fill(card_surface())
        .stroke(theme::hairline())
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            add_contents(ui);
        });
}
