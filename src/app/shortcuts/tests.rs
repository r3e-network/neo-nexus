use eframe::egui;

use super::{
    nodes::shifted_node_index,
    views::{next_view, numbered_view_shortcut, previous_view},
    View,
};

mod labels;

#[test]
fn numbered_view_shortcuts_select_primary_workspaces() {
    assert_eq!(numbered_view_shortcut(egui::Key::Num1), Some(View::Summary));
    assert_eq!(numbered_view_shortcut(egui::Key::Num8), Some(View::Wallets));
    assert_eq!(numbered_view_shortcut(egui::Key::Num9), Some(View::Nodes));
    assert_eq!(numbered_view_shortcut(egui::Key::Num0), None);
}

#[test]
fn view_cycling_wraps_fixed_native_workspace_tabs() {
    assert_eq!(next_view(View::Summary), View::Operations);
    assert_eq!(previous_view(View::Summary), View::Logs);
    assert_eq!(next_view(View::Logs), View::Summary);
}

#[test]
fn node_navigation_clamps_to_available_inventory() {
    assert_eq!(shifted_node_index(None, 0, 1), None);
    assert_eq!(shifted_node_index(None, 4, 1), Some(0));
    assert_eq!(shifted_node_index(None, 4, -1), Some(3));
    assert_eq!(shifted_node_index(Some(2), 4, 1), Some(3));
    assert_eq!(shifted_node_index(Some(3), 4, 1), Some(3));
    assert_eq!(shifted_node_index(Some(0), 4, -1), Some(0));
    assert_eq!(shifted_node_index(Some(3), 4, -7), Some(0));
}
