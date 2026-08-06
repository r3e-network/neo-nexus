use eframe::egui;

use crate::app::theme;

/// Height of a segment button. Kept a touch shorter than a standalone button so
/// a section switcher reads as chrome rather than as a row of primary actions.
const SEGMENT_HEIGHT: f32 = 28.0;

/// Horizontal padding inside a segment, either side of its label.
const SEGMENT_PAD_X: f32 = 12.0;

/// Segmented control: segments inside a recessed track. The active segment
/// lifts out of the track as a raised card-coloured pill with a hairline edge
/// and an accent label — the same "selected pill" language the sidebar and
/// filter chips use — so selection never paints a solid accent block behind
/// body copy. Returns `true` when the selection changes this frame.
///
/// The control reflows. Equal-width columns are used only while every label
/// fits; once they do not, the segments wrap onto as many lines as they need at
/// their natural widths. Six equal columns in a narrow workspace produced tab
/// labels like "Read…" and "Jour…", which is worse than a two-line switcher.
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

    let equal_columns = fits_equal_columns(ui, segments);
    let mut changed = false;
    egui::Frame::new()
        .fill(theme::track_surface())
        .stroke(theme::hairline())
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::same(3))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            // A segment label is a single word sized to fit by `equal_columns`.
            // Letting it wrap instead would silently split it mid-word
            // ("Plugin/s"); extending makes a bad fit fail the containment
            // contract loudly instead of shipping a broken tab strip.
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
            if equal_columns {
                ui.columns(segments.len(), |columns| {
                    for (index, (column, label)) in columns.iter_mut().zip(segments).enumerate() {
                        let width = column.available_width();
                        if segment(column, label, index == *selected, Some(width)) {
                            *selected = index;
                            changed = true;
                        }
                    }
                });
                return;
            }
            ui.horizontal_wrapped(|ui| {
                for (index, label) in segments.iter().enumerate() {
                    if segment(ui, label, index == *selected, None) {
                        *selected = index;
                        changed = true;
                    }
                }
            });
        });
    changed
}

/// Whether every label still reads in full when the track is split into equal
/// columns. Measured against the real font, not guessed from character counts.
fn fits_equal_columns(ui: &egui::Ui, segments: &[&str]) -> bool {
    let widest = segments
        .iter()
        .map(|label| label_width(ui, label))
        .fold(0.0_f32, f32::max);
    // Account for the frame margin and the gap egui puts between columns, then
    // require the widest label to clear its padding outright. A segment that is
    // one point short does not shrink its text, it wraps it mid-word.
    let gaps = 2.0 * (segments.len().saturating_sub(1)) as f32;
    let track = ui.available_width() - 6.0 - gaps;
    let per_column = track / segments.len() as f32;
    per_column >= widest + SEGMENT_PAD_X * 2.0
}

fn label_width(ui: &egui::Ui, label: &str) -> f32 {
    ui.painter()
        .layout_no_wrap(label.to_string(), theme::body_font(), theme::text())
        .size()
        .x
}

/// Draws one segment. Returns `true` when it was clicked while not already
/// selected. `width` fixes the segment to a column; `None` sizes it to its own
/// label for the wrapped layout.
fn segment(ui: &mut egui::Ui, label: &str, is_selected: bool, width: Option<f32>) -> bool {
    let text = if is_selected {
        theme::body(label).color(theme::accent_text()).strong()
    } else {
        theme::body(label).color(theme::muted_text())
    };
    let min_width = width.unwrap_or_else(|| label_width(ui, label) + SEGMENT_PAD_X * 2.0);
    let mut button = egui::Button::new(text)
        .corner_radius(egui::CornerRadius::same(7))
        .min_size(egui::vec2(min_width, SEGMENT_HEIGHT));
    button = if is_selected {
        button
            .fill(theme::card_surface())
            .stroke(theme::hairline_strong())
    } else {
        button
            .fill(egui::Color32::TRANSPARENT)
            .stroke(egui::Stroke::NONE)
    };
    ui.add(button).clicked() && !is_selected
}
