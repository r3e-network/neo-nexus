//! Vertical form primitives for the v3 workbench (Node Studio and later forms).

use eframe::egui;

use crate::app::theme;

use super::layout::card_frame;

/// Height of a form editor. Two points shorter than a standalone control: a
/// form group stacks four of these, and the surfaces they live on do not
/// scroll, so every point of vertical rhythm is a point of content that either
/// fits inside the panel or is never seen.
const FIELD_HEIGHT: f32 = 26.0;

/// Gap between a field's label and its editor.
const LABEL_GAP: f32 = 2.0;

/// Label-above text field (form rows that need more breathing room than the
/// compact horizontal `labeled_text` helper).
pub(in crate::app) fn field_text(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        ui.label(theme::label_caption(label));
        ui.add_space(LABEL_GAP);
        ui.add_sized(
            [ui.available_width().max(120.0), FIELD_HEIGHT],
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
        ui.spacing_mut().item_spacing.y = 0.0;
        ui.label(theme::label_caption(label));
        ui.add_space(LABEL_GAP);
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

/// Soft inset surface for nested form groups without full card chrome. Filled
/// with the recessed track tone, not the card tone: a form group always sits
/// *inside* a card, so a card-coloured fill would be invisible now that cards
/// no longer cast a shadow to separate the layers.
pub(in crate::app) fn form_group(
    ui: &mut egui::Ui,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    ui.label(theme::label_caption(title));
    ui.add_space(theme::XS);
    egui::Frame::new()
        .fill(theme::track_surface())
        .stroke(theme::hairline())
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            add_contents(ui);
        });
}
