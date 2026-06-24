use eframe::egui;

use crate::app::theme::card_surface;

/// macOS-style segmented control: a single rounded, hairline-bordered pill that
/// holds equal-width selectable segments. The active segment lifts with the
/// theme selection colour. Returns `true` when the selection changes this frame
/// so callers can react (e.g. reset a sub-page index).
pub(in crate::app) fn segmented_control(
    ui: &mut egui::Ui,
    segments: &[&str],
    selected: &mut usize,
) -> bool {
    if segments.is_empty() {
        return false;
    }
    if *selected >= segments.len() {
        *selected = segments.len() - 1;
    }

    let mut changed = false;
    let stroke = ui.style().visuals.widgets.noninteractive.bg_stroke;
    egui::Frame::new()
        .fill(card_surface())
        .stroke(stroke)
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::same(3))
        .show(ui, |ui| {
            ui.columns(segments.len(), |columns| {
                for (index, (column, label)) in columns.iter_mut().zip(segments).enumerate() {
                    let is_selected = index == *selected;
                    let width = column.available_width();
                    let response = column
                        .add_sized([width, 26.0], egui::Button::selectable(is_selected, *label));
                    if response.clicked() && !is_selected {
                        *selected = index;
                        changed = true;
                    }
                }
            });
        });
    changed
}
