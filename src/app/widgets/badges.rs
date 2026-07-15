use eframe::egui::{self, Color32};

use crate::app::{
    domain::{CheckSeverity, NodeStatus},
    theme::{self, status_color},
};

/// Compact status pill used in lists and headers (Running / Stopped / …).
pub(in crate::app) fn status_badge(ui: &mut egui::Ui, status: NodeStatus) {
    let color = status_color(status);
    badge_pill(ui, status.label(), color, status_fill(color));
}

/// Severity pill for readiness / diagnostics rows.
pub(in crate::app) fn severity_badge(ui: &mut egui::Ui, severity: CheckSeverity) {
    let color = severity_color(severity);
    badge_pill(ui, severity.label(), color, status_fill(color));
}

/// Generic labelled badge with an explicit colour (type/network chips, etc.).
pub(in crate::app) fn text_badge(ui: &mut egui::Ui, text: &str, color: Color32) {
    badge_pill(ui, text, color, status_fill(color));
}

fn badge_pill(ui: &mut egui::Ui, text: &str, fg: Color32, bg: Color32) {
    let label = egui::RichText::new(text).size(11.0).color(fg).strong();
    egui::Frame::new()
        .fill(bg)
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.label(label);
        });
}

fn status_fill(color: Color32) -> Color32 {
    // Soft tint of the status hue so badges read as chips, not solid blocks.
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 36)
}

pub(in crate::app) fn severity_color(severity: CheckSeverity) -> Color32 {
    match severity {
        CheckSeverity::Pass => theme::success(),
        CheckSeverity::Info => theme::info(),
        CheckSeverity::Warning => theme::warning(),
        CheckSeverity::Critical => theme::danger(),
    }
}

/// Status indicator dot for dense rows (inventory, fleet).
pub(in crate::app) fn status_dot(ui: &mut egui::Ui, status: NodeStatus) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        ui.painter()
            .circle_filled(rect.center(), 4.0, status_color(status));
    }
}
