use eframe::egui;

use crate::app::theme::{self, card_surface};

/// macOS-style segmented control: equal-width segments in a hairline pill.
/// The active segment uses the accent fill so it is unmistakable. Returns
/// `true` when the selection changes this frame.
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
    let stroke = theme::hairline();
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
                    let text = if is_selected {
                        theme::body(*label).color(theme::on_accent()).strong()
                    } else {
                        theme::body(*label)
                    };
                    let mut button = egui::Button::new(text)
                        .corner_radius(egui::CornerRadius::same(6))
                        .min_size(egui::vec2(width, 28.0));
                    if is_selected {
                        button = button.fill(theme::accent()).stroke(egui::Stroke::NONE);
                    } else {
                        button = button.fill(egui::Color32::TRANSPARENT);
                    }
                    let response = column.add(button);
                    if response.clicked() && !is_selected {
                        *selected = index;
                        changed = true;
                    }
                }
            });
        });
    changed
}
