use eframe::egui;

use crate::app::theme;

pub(super) struct OverviewLayout {
    pub(super) left_width: f32,
    pub(super) right_width: f32,
    pub(super) actions_height: f32,
    pub(super) fleet_height: f32,
    pub(super) height: f32,
    pub(super) gap: f32,
}

pub(super) fn overview_layout(available: egui::Vec2) -> OverviewLayout {
    let gap = theme::SM;
    // Left selection column + right triage column (next actions over fleet).
    // Widths are fractional so three chrome panels never force overflow.
    let left_width = (available.x * 0.38).clamp(available.x * 0.28, available.x * 0.50);
    let right_width = (available.x - left_width - gap).max(available.x * 0.42);
    // Give the action queue priority: operators open Home to see what to do next.
    let actions_height = (available.y * 0.48).clamp(180.0, 280.0);
    let fleet_height = (available.y - actions_height - gap).max(180.0);

    OverviewLayout {
        left_width,
        right_width,
        actions_height,
        fleet_height,
        height: available.y,
        gap,
    }
}
