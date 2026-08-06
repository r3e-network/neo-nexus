use eframe::egui;

use crate::app::theme::{self, card_surface};

/// Corner radius shared by every card-class surface, so cards, panels, and the
/// containers built on top of them round identically.
pub(in crate::app) const CARD_CORNER_RADIUS: u8 = 12;

/// Shared surface for cards and panels: a filled, softly rounded container with
/// a hairline border and **no** drop shadow. Elevation in this workbench is
/// carried by the fill tiers (`window_fill` < `panel_fill` < `card_fill`) plus
/// the hairline edge, which reads as a crisp, calm document surface rather than
/// a floating chip.
#[allow(dead_code)] // used by form kit; re-export kept for shared card chrome
pub(in crate::app) fn card_frame(_style: &egui::Style) -> egui::Frame {
    egui::Frame::new()
        .fill(card_surface())
        .stroke(theme::hairline())
        .corner_radius(egui::CornerRadius::same(CARD_CORNER_RADIUS))
        .inner_margin(egui::Margin::symmetric(14, 12))
}

/// Narrowest a metric tile may become before its eyebrow and caption stop being
/// readable words and start being ellipses.
const MIN_TILE_WIDTH: f32 = 118.0;

/// Render a row of equal-width metric cards. Columns share the available width
/// evenly, so a four-up summary stays aligned regardless of value lengths.
///
/// The row reflows: when the workspace is too narrow to give every tile
/// [`MIN_TILE_WIDTH`], the tiles wrap onto as many rows as needed rather than
/// shrinking into a strip of truncated stubs.
pub(in crate::app) fn metric_row(ui: &mut egui::Ui, tiles: &[(&str, &str, &str)]) {
    if tiles.is_empty() {
        return;
    }
    let per_row = tiles_per_row(ui.available_width(), tiles.len());
    for (index, chunk) in tiles.chunks(per_row).enumerate() {
        if index > 0 {
            ui.add_space(theme::SM);
        }
        // Every row is laid out on the same column count so tiles stay aligned
        // in a grid even when the last row is short.
        ui.columns(per_row, |columns| {
            for (column, &(title, value, caption)) in columns.iter_mut().zip(chunk) {
                metric_tile(column, title, value, caption);
            }
        });
    }
}

/// How many tiles fit on one row at `available` width, never fewer than one and
/// never more than there are tiles. The count is then balanced across the rows
/// it needs, so four tiles in a three-wide space become 2 + 2 rather than a
/// crowded 3 beside a lonely 1.
fn tiles_per_row(available: f32, count: usize) -> usize {
    let fits = ((available / MIN_TILE_WIDTH).floor().max(1.0) as usize).clamp(1, count.max(1));
    let rows = count.div_ceil(fits.max(1)).max(1);
    count.div_ceil(rows).max(1)
}

#[cfg(test)]
#[path = "../../../tests/unit/app/widgets/layout/tests.rs"]
mod tests;

fn metric_tile(ui: &mut egui::Ui, title: &str, value: &str, caption: &str) {
    card_frame(ui.style()).show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        // A fixed-width cell in a column row. Without this the eyebrow wraps one
        // character per line the moment the tile is narrower than its longest
        // word, which turns a summary strip into a column of single letters.
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
        ui.spacing_mut().item_spacing.y = 2.0;
        ui.vertical(|ui| {
            ui.label(theme::label_caption(title));
            ui.label(theme::metric_value(value));
        })
        // The caption said the same thing as the eyebrow — "HEALTH / 0% /
        // running nodes" labels one number twice. It becomes hover text, and
        // the strip loses a third of its height on every page that carries one.
        .response
        .on_hover_text(caption);
    });
}

/// Compact labelled stat for dense side panels: a caption over a value, with no
/// card chrome (the surrounding panel provides the surface). Used where a full
/// metric tile would be too heavy but a plain fact row lacks presence — e.g. the
/// inventory's Total/Running/Stopped/Visible counts.
pub(in crate::app) fn mini_stat(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.vertical(|ui| {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
        ui.label(theme::label_caption(label));
        ui.label(theme::section_title(value));
    });
}

/// A titled card that claims the full remaining height, for surfaces that own
/// their region. Thin wrapper over the shared card chrome so `panel` and `card`
/// cannot drift apart visually.
pub(in crate::app) fn panel(
    ui: &mut egui::Ui,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    super::card::stretch_card(ui, title, add_contents);
}

/// Header row for an `egui::Grid` data table: renders each column label with the
/// shared macOS-style column-header treatment and closes the row.
pub(in crate::app) fn grid_header(ui: &mut egui::Ui, headers: &[&str]) {
    for header in headers {
        ui.label(theme::column_header(*header));
    }
    ui.end_row();
}

pub(in crate::app) fn empty_state(ui: &mut egui::Ui, title: &str, message: &str) {
    empty_state_with_action(ui, title, message, None);
}

/// Empty state with an optional primary CTA. Returns `true` when the CTA is
/// clicked so callers can route to create/import flows.
pub(in crate::app) fn empty_state_with_action(
    ui: &mut egui::Ui,
    title: &str,
    message: &str,
    action: Option<&str>,
) -> bool {
    let mut clicked = false;
    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() * 0.28);
        // A muted tray pictogram anchors the empty state with a focal mark, the
        // way a macOS empty list does, so the guidance reads as intentional
        // rather than as bare placeholder words. Rendered at the metric-value
        // size (24pt, on the type scale) and the muted tone so it stays a quiet
        // presence above the guidance, not an alarming one.
        ui.label(theme::metric_value(theme::empty_glyph()).color(theme::muted_text()));
        ui.add_space(theme::SM);
        ui.label(theme::page_title(title));
        ui.add_space(theme::XS);
        ui.label(theme::muted_body(message));
        if let Some(label) = action {
            ui.add_space(theme::MD);
            if super::controls::primary_button(ui, label).clicked() {
                clicked = true;
            }
        }
    });
    clicked
}
