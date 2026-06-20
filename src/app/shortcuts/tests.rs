use eframe::egui;

use super::{
    labels::{
        node_menu_shortcuts, primary_action_shortcuts, shortcut_command_label, shortcut_menu_label,
        shortcut_toolbar_label, view_menu_shortcuts, view_shortcut_hint, workspace_menu_shortcuts,
    },
    nodes::shifted_node_index,
    views::{next_view, numbered_view_shortcut, previous_view},
    AppShortcut, View,
};

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

#[test]
fn menu_shortcut_labels_make_native_commands_discoverable() {
    assert_eq!(
        shortcut_menu_label(AppShortcut::ReloadWorkspace),
        "Reload    Cmd+R / F5"
    );
    assert_eq!(
        shortcut_menu_label(AppShortcut::RestartSelectedNode),
        "Restart selected    Cmd+Shift+Enter"
    );
    assert_eq!(
        shortcut_menu_label(AppShortcut::PreviousView),
        "Previous view    Cmd+["
    );
}

#[test]
fn view_shortcut_hints_cover_numbered_primary_workspaces() {
    assert_eq!(view_shortcut_hint(View::Summary), Some("Cmd+1"));
    assert_eq!(view_shortcut_hint(View::Wallets), Some("Cmd+8"));
    assert_eq!(view_shortcut_hint(View::Nodes), Some("Cmd+9"));
    assert_eq!(view_shortcut_hint(View::Roles), None);
}

#[test]
fn node_menu_shortcuts_cover_navigation_and_lifecycle_commands() {
    assert_eq!(
        node_menu_shortcuts(),
        [
            AppShortcut::NewNode,
            AppShortcut::PreviousNode,
            AppShortcut::NextNode,
            AppShortcut::PreviousNodePage,
            AppShortcut::NextNodePage,
            AppShortcut::FirstNode,
            AppShortcut::LastNode,
            AppShortcut::StartSelectedNode,
            AppShortcut::StopSelectedNode,
            AppShortcut::RestartSelectedNode,
        ]
    );
    assert_eq!(
        shortcut_command_label(AppShortcut::StartSelectedNode),
        "Start selected"
    );
    assert_eq!(
        shortcut_command_label(AppShortcut::StopSelectedNode),
        "Stop selected"
    );
}

#[test]
fn primary_action_shortcuts_match_fixed_header_buttons() {
    assert_eq!(
        primary_action_shortcuts(),
        [
            AppShortcut::NewNode,
            AppShortcut::ReloadWorkspace,
            AppShortcut::StartSelectedNode,
            AppShortcut::StopSelectedNode,
            AppShortcut::RestartSelectedNode,
        ]
    );
    assert_eq!(shortcut_toolbar_label(AppShortcut::NewNode), "New Node");
    assert_eq!(
        shortcut_toolbar_label(AppShortcut::ReloadWorkspace),
        "Reload"
    );
    assert_eq!(
        shortcut_toolbar_label(AppShortcut::StartSelectedNode),
        "Start"
    );
    assert_eq!(
        shortcut_toolbar_label(AppShortcut::StopSelectedNode),
        "Stop"
    );
}

#[test]
fn view_menu_shortcuts_surface_cycle_commands_before_workspace_targets() {
    let shortcuts = view_menu_shortcuts();
    assert_eq!(shortcuts[0], AppShortcut::PreviousView);
    assert_eq!(shortcuts[1], AppShortcut::NextView);
    assert_eq!(shortcuts[2], AppShortcut::SelectView(View::Summary));
    assert_eq!(shortcuts[10], AppShortcut::SelectView(View::Nodes));
    assert_eq!(
        shortcuts[shortcuts.len() - 1],
        AppShortcut::SelectView(View::Logs)
    );
    assert_eq!(shortcuts.len(), View::ALL.len() + 2);
}

#[test]
fn workspace_menu_shortcuts_cover_workspace_level_commands() {
    assert_eq!(
        workspace_menu_shortcuts(),
        [
            AppShortcut::ReloadWorkspace,
            AppShortcut::SelectView(View::Summary),
            AppShortcut::SelectView(View::Settings),
        ]
    );
    assert_eq!(
        shortcut_command_label(AppShortcut::ReloadWorkspace),
        "Reload"
    );
    assert_eq!(
        shortcut_command_label(AppShortcut::SelectView(View::Settings)),
        "Settings"
    );
}
