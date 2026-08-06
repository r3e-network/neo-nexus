use eframe::egui;

use crate::app::theme;

/// The primary, accent-filled confirm action of a form. Use for the single
/// dominant action (Save / Create / Import); leave secondary actions as plain
/// bordered buttons.
///
/// The accent is applied through the widget visuals rather than `Button::fill`
/// so the button actually responds to the pointer: `accent` at rest, the deeper
/// `accent_hover` on hover and while pressed. A hard `fill` would freeze it in
/// one colour and leave the app's most important control feeling dead.
pub(in crate::app) fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.scope(|ui| {
        let widgets = &mut ui.style_mut().visuals.widgets;
        for (state, fill) in [
            (&mut widgets.inactive, theme::accent()),
            (&mut widgets.hovered, theme::accent_hover()),
            (&mut widgets.active, theme::accent_hover()),
        ] {
            state.bg_fill = fill;
            state.weak_bg_fill = fill;
            state.bg_stroke = egui::Stroke::NONE;
        }
        ui.add(
            egui::Button::new(egui::RichText::new(text).color(theme::on_accent()).strong())
                .corner_radius(egui::CornerRadius::same(9))
                .min_size(egui::vec2(100.0, 30.0)),
        )
    })
    .inner
}

/// A secondary action button: a plain bordered button that shares the
/// `primary_button` minimum height so a row of actions (e.g. Start / Stop /
/// Edit) reads as one coherent, evenly-sized group rather than a mix of
/// egui-default and accent buttons. Pair with `primary_button` for the
/// dominant/confirm action.
pub(in crate::app) fn secondary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(text)
            .corner_radius(egui::CornerRadius::same(8))
            .min_size(egui::vec2(76.0, 30.0)),
    )
}

/// Same as `secondary_button` but disabled when `enabled` is false, so a
/// lifecycle action (e.g. Stop while stopped) keeps its consistent size while
/// greyed out rather than collapsing or shifting the row.
pub(in crate::app) fn secondary_button_enabled(
    ui: &mut egui::Ui,
    text: &str,
    enabled: bool,
) -> egui::Response {
    ui.add_enabled(
        enabled,
        egui::Button::new(text)
            .corner_radius(egui::CornerRadius::same(8))
            .min_size(egui::vec2(76.0, 30.0)),
    )
}

/// Wraps a row of mutually-exclusive filter chips in a recessed track so
/// single-select filters read as one segmented control rather than a loose row
/// of toggles. The track must be *darker* than the surrounding surface, because
/// the selected chip is now a card-coloured pill — on a card-coloured track it
/// would be white on white and the selection would vanish.
pub(in crate::app) fn chip_pill(ui: &mut egui::Ui, add_chips: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(theme::track_surface())
        .stroke(egui::Stroke::NONE)
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(3, 2))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            // Wrapped, not fixed: a five-way status filter is wider than the
            // narrow inventory column, and an unwrapped row silently widened
            // the whole panel to fit — squeezing the central workspace.
            ui.horizontal_wrapped(add_chips);
        });
}

pub(in crate::app) fn pagination_bar(
    ui: &mut egui::Ui,
    page: &mut usize,
    total_pages: usize,
    item_count: usize,
) {
    ui.horizontal(|ui| {
        ui.label(theme::muted_body(format!("{item_count} items")));
        ui.separator();
        ui.label(theme::muted_body(format!(
            "{}/{}",
            *page + 1,
            total_pages.max(1)
        )));
        ui.add_space(theme::SM);
        if secondary_button_enabled(ui, "Previous", *page > 0).clicked() {
            *page -= 1;
        }
        if secondary_button_enabled(ui, "Next", *page + 1 < total_pages).clicked() {
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
