use super::{overview_layout, shows_fleet_snapshot, shows_selection_column};
use crate::app::widgets::MIN_COLUMN_WIDTH;
use eframe::egui;

#[test]
fn columns_and_gap_never_exceed_the_available_width() {
    for width in [320.0_f32, 480.0, 620.0, 900.0, 1280.0] {
        let layout = overview_layout(egui::vec2(width, 700.0), true);
        let used = layout.left_width + layout.gap + layout.right_width;
        assert!(
            used <= width + 0.5,
            "overview columns use {used} of {width} available",
        );
    }
}

/// The two stacked cards must fit whatever height they are given, including
/// heights far below what a comfortable dashboard would like. Absolute floors
/// here used to force 368pt of cards into panels shorter than that, pushing the
/// fleet snapshot below the edge of a surface that does not scroll.
#[test]
fn stacked_panel_heights_fit_the_available_height() {
    for height in [120.0_f32, 240.0, 360.0, 700.0, 1400.0] {
        let layout = overview_layout(egui::vec2(900.0, height), true);
        let used = layout.actions_height + layout.gap + layout.fleet_height;
        assert!(
            used <= height + 0.5,
            "overview rows use {used} of {height} available",
        );
        assert!(layout.actions_height > layout.fleet_height);
    }
}

#[test]
fn the_action_queue_takes_the_whole_column_when_the_fleet_card_is_dropped() {
    let layout = overview_layout(egui::vec2(900.0, 700.0), false);
    assert_eq!(layout.fleet_height, 0.0);
    assert!((layout.actions_height - 700.0).abs() < 0.5);
}

/// The inventory column already lists the fleet with the same row widget, so
/// Home must not restate it beside the same list.
#[test]
fn the_fleet_snapshot_yields_to_the_inventory_column() {
    assert!(!shows_fleet_snapshot(true));
    assert!(shows_fleet_snapshot(false));
}

#[test]
fn the_selection_column_is_dropped_when_the_inspector_shows_the_same_facts() {
    assert!(!shows_selection_column(1000.0, true));
    assert!(shows_selection_column(1000.0, false));
}

#[test]
fn the_selection_column_is_dropped_below_the_two_column_width() {
    let threshold = MIN_COLUMN_WIDTH * 2.0 + crate::app::theme::SM;
    assert!(!shows_selection_column(threshold - 1.0, false));
    assert!(shows_selection_column(threshold, false));
}
