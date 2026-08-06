use eframe::egui;

use crate::app::{theme, widgets::fits_side_by_side};

pub(super) struct OverviewLayout {
    pub(super) left_width: f32,
    pub(super) right_width: f32,
    pub(super) actions_height: f32,
    pub(super) fleet_height: f32,
    pub(super) height: f32,
    pub(super) gap: f32,
}

/// Share of the triage column the action queue gets when it shares the column
/// with the fleet snapshot. Operators open Home to see what to do next, so the
/// queue takes the larger half.
const ACTIONS_SHARE: f32 = 0.55;

pub(super) fn overview_layout(available: egui::Vec2, shows_fleet: bool) -> OverviewLayout {
    let gap = theme::SM;
    // Left selection column + right triage column (next actions over fleet).
    // Widths are fractional so three chrome panels never force overflow.
    let left_width = (available.x * 0.38).clamp(available.x * 0.28, available.x * 0.50);
    let right_width = (available.x - left_width - gap).max(available.x * 0.42);

    // Heights are a pure split of the room available, with no absolute floor.
    // A floor is worse than useless here: when the panel is shorter than the
    // floors add up to, they force the two cards past the bottom of a surface
    // that does not scroll, and the second card is simply never seen.
    let (actions_height, fleet_height) = if shows_fleet {
        let usable = (available.y - gap).max(0.0);
        let actions = usable * ACTIONS_SHARE;
        (actions, usable - actions)
    } else {
        (available.y.max(0.0), 0.0)
    };

    OverviewLayout {
        left_width,
        right_width,
        actions_height,
        fleet_height,
        height: available.y,
        gap,
    }
}

/// True when the overview has both the room and the reason to show its own
/// selection column. With the inspector open that column is a second copy of
/// the same facts, so it is dropped and the width handed to the triage column.
pub(super) fn shows_selection_column(available_width: f32, inspector_visible: bool) -> bool {
    !inspector_visible && fits_side_by_side(available_width)
}

/// True when the fleet snapshot earns its space. It does not when the inventory
/// column is already on screen listing the same nodes with the same row widget:
/// showing both spends half of Home restating what is two centimetres to the
/// left, and squeezes the action queue — the one thing Home is for.
pub(super) fn shows_fleet_snapshot(inventory_visible: bool) -> bool {
    !inventory_visible
}

#[cfg(test)]
#[path = "../../../../tests/unit/app/views/overview/layout/tests.rs"]
mod tests;
