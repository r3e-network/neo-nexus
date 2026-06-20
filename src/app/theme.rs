use eframe::egui;

use crate::types::NodeStatus;

pub(super) fn configure_style(context: &egui::Context) {
    let mut style = (*context.style()).clone();
    style.visuals.panel_fill = egui::Color32::from_rgb(238, 242, 247);
    style.visuals.window_fill = egui::Color32::from_rgb(248, 250, 252);
    style.visuals.extreme_bg_color = egui::Color32::from_rgb(226, 232, 240);
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(14, 116, 144);
    style.visuals.selection.stroke.color = egui::Color32::WHITE;
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
    context.set_style(style);
}

pub(super) fn accent() -> egui::Color32 {
    egui::Color32::from_rgb(14, 116, 144)
}

pub(super) fn muted_text() -> egui::Color32 {
    egui::Color32::from_rgb(71, 85, 105)
}

pub(super) fn panel_fill() -> egui::Color32 {
    egui::Color32::from_rgb(255, 255, 255)
}

pub(super) fn status_color(status: NodeStatus) -> egui::Color32 {
    match status {
        NodeStatus::Running => egui::Color32::from_rgb(21, 128, 61),
        NodeStatus::Starting => egui::Color32::from_rgb(202, 138, 4),
        NodeStatus::Stopped => egui::Color32::from_rgb(75, 85, 99),
        NodeStatus::Error => egui::Color32::from_rgb(185, 28, 28),
    }
}
