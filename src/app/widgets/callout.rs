use eframe::egui::{self, Color32};

use crate::app::theme::{self, card_surface};

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
pub(in crate::app) fn callout(ui: &mut egui::Ui, kind: CalloutKind, title: &str, body: &str) {
    let accent = kind.accent();
    egui::Frame::new()
        .fill(card_surface())
        .stroke(egui::Stroke::new(1.0, theme::hairline().color))
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::ZERO)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (_, rail_rect) = ui.allocate_space(egui::vec2(4.0, 0.0));
                let height = ui.min_rect().height().max(44.0);
                let rail = egui::Rect::from_min_size(
                    rail_rect.min,
                    egui::vec2(4.0, height),
                );
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
