use eframe::egui;

use crate::app::theme;

pub(super) struct AlertPaneLayout {
    pub(super) policy_width: f32,
    pub(super) history_width: f32,
    pub(super) height: f32,
    pub(super) gap: f32,
}

/// Splits the row between the editor and the delivery history.
///
/// The minimum widths are floors on what stays readable, not promises: floored
/// independently they summed to more than the row and the pair painted ~86pt
/// past the column that clips them. When the room is not there, both shrink
/// proportionally instead — a cramped pane is recoverable, one painted outside
/// the clip is not.
pub(super) fn alert_pane_layout(available: egui::Vec2) -> AlertPaneLayout {
    let gap = theme::SM;
    let usable = (available.x - gap).max(0.0);
    let (policy_min, history_min) = (340.0_f32, 360.0);

    let (policy_width, history_width) = if usable >= policy_min + history_min {
        let policy = (available.x * 0.38).clamp(policy_min, 500.0);
        (policy, usable - policy)
    } else {
        let share = usable / (policy_min + history_min);
        (policy_min * share, history_min * share)
    };

    AlertPaneLayout {
        policy_width,
        history_width,
        height: available.y,
        gap,
    }
}
