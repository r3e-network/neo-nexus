//! Two-column label/value grid for side panels and summary cards.
//!
//! Cells are separated by hairlines rather than by striped fills: banding reads
//! as a data table, hairlines read as a specification sheet, and the latter is
//! what an operator scanning ports, versions, and process facts actually wants.

use eframe::egui;

use crate::app::theme;

/// Vertical padding inside one cell.
const CELL_PAD_Y: f32 = 4.0;

/// Renders `pairs` as a two-column grid of label-over-value cells with hairline
/// separators between rows. An odd final pair spans the row on its own.
pub(in crate::app) fn metric_grid(ui: &mut egui::Ui, pairs: &[(&str, String)]) {
    if pairs.is_empty() {
        return;
    }
    let rows = pairs.chunks(2);
    let row_count = rows.len();
    for (index, row) in pairs.chunks(2).enumerate() {
        ui.spacing_mut().item_spacing.y = 0.0;
        ui.columns(2, |columns| {
            for (column, (label, value)) in columns.iter_mut().zip(row) {
                cell(column, label, value);
            }
        });
        if index + 1 < row_count {
            super::divider::hr_tight(ui);
        }
    }
}

fn cell(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
    ui.vertical(|ui| {
        ui.add_space(CELL_PAD_Y);
        ui.label(theme::caption(label));
        ui.add_space(2.0);
        ui.label(theme::body(value).strong());
        ui.add_space(CELL_PAD_Y);
    });
}
