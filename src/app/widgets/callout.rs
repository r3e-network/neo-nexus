use eframe::egui::{self, Color32};

use crate::app::theme;

/// Width of the coloured leading edge. Shared with the sidebar's selected-nav
/// marker so the workbench has exactly one left-bar width.
pub(in crate::app) const RAIL_WIDTH: f32 = 3.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Success reserved for later surfaces
pub(in crate::app) enum CalloutKind {
    Info,
    Success,
    Warning,
    Danger,
}

impl CalloutKind {
    fn accent(self) -> Color32 {
        match self {
            Self::Info => theme::info(),
            Self::Success => theme::success(),
            Self::Warning => theme::warning(),
            Self::Danger => theme::danger(),
        }
    }
}

/// Inline guidance strip with a coloured leading edge (warnings, next steps).
/// The body is washed with a low-alpha tint of the kind hue so the strip reads
/// as a semantic surface at a glance, the way the status chips do, instead of
/// relying on a 3pt rail alone to carry the meaning.
pub(in crate::app) fn callout(ui: &mut egui::Ui, kind: CalloutKind, title: &str, body: &str) {
    let accent = kind.accent();
    egui::Frame::new()
        .fill(tint(accent))
        .stroke(egui::Stroke::new(1.0_f32, tint_border(accent)))
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::ZERO)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (_, rail_rect) = ui.allocate_space(egui::vec2(RAIL_WIDTH, 0.0));
                let height = ui.min_rect().height().max(44.0);
                let rail = egui::Rect::from_min_size(rail_rect.min, egui::vec2(RAIL_WIDTH, height));
                ui.painter()
                    .rect_filled(rail, egui::CornerRadius::same(2), accent);

                ui.add_space(theme::SM);
                ui.vertical(|ui| {
                    ui.add_space(theme::SM);
                    ui.label(theme::body(title).strong().color(accent));
                    if !body.is_empty() {
                        ui.add_space(theme::XS);
                        ui.label(theme::muted_body(body));
                    }
                    ui.add_space(theme::SM);
                });
            });
        });
}

/// A very low-alpha wash of the kind hue for the callout body.
fn tint(color: Color32) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 22)
}

/// A slightly stronger wash for the callout edge, so the strip still has a
/// definite boundary without a hard coloured border.
fn tint_border(color: Color32) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 60)
}
