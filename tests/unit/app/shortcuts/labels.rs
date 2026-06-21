use super::super::{
    labels::{
        alternate_navigation_modifier_label, command_modifier_label, node_menu_shortcuts,
        primary_action_shortcuts, shortcut_command_label, shortcut_hint_for_modifier,
        shortcut_hint_for_modifiers, shortcut_menu_label, shortcut_menu_label_for_modifier,
        shortcut_menu_label_for_modifiers, shortcut_toolbar_label, view_menu_label_for_modifier,
        view_menu_shortcuts, view_shortcut_hint, view_shortcut_hint_for_modifier,
        workspace_menu_shortcuts,
    },
    AppShortcut, View,
};

#[test]
fn menu_shortcut_labels_make_native_commands_discoverable() {
    let modifier = command_modifier_label();
    let alternate = alternate_navigation_modifier_label();
    assert_eq!(
        shortcut_menu_label(AppShortcut::ReloadWorkspace),
        format!("Reload    {modifier}+R / F5")
    );
    assert_eq!(
        shortcut_menu_label(AppShortcut::RestartSelectedNode),
        format!("Restart selected    {modifier}+Shift+Enter")
    );
    assert_eq!(
        shortcut_menu_label(AppShortcut::PreviousView),
        format!("Previous view    {modifier}+[")
    );
    assert_eq!(
        shortcut_menu_label(AppShortcut::NextNode),
        format!("Next node    {alternate}+Down")
    );
}

#[test]
fn shortcut_labels_render_with_supplied_command_modifier() {
    assert_eq!(
        shortcut_menu_label_for_modifier(AppShortcut::ReloadWorkspace, "Ctrl"),
        "Reload    Ctrl+R / F5"
    );
    assert_eq!(
        shortcut_menu_label_for_modifier(AppShortcut::RestartSelectedNode, "Ctrl"),
        "Restart selected    Ctrl+Shift+Enter"
    );
    assert_eq!(
        view_menu_label_for_modifier(View::Nodes, "Ctrl"),
        "Nodes    Ctrl+9"
    );
    assert_eq!(
        shortcut_hint_for_modifier(AppShortcut::PreviousView, "Ctrl"),
        Some("Ctrl+[".to_string())
    );
    assert_eq!(
        view_shortcut_hint_for_modifier(View::Wallets, "Ctrl"),
        Some("Ctrl+8".to_string())
    );
}

#[test]
fn inventory_navigation_labels_render_with_supplied_alternate_modifier() {
    assert_eq!(
        shortcut_menu_label_for_modifiers(AppShortcut::PreviousNode, "Cmd", "Option"),
        "Previous node    Option+Up"
    );
    assert_eq!(
        shortcut_menu_label_for_modifiers(AppShortcut::NextNodePage, "Cmd", "Option"),
        "Next node page    Option+PageDown"
    );
    assert_eq!(
        shortcut_hint_for_modifiers(AppShortcut::FirstNode, "Cmd", "Option"),
        Some("Option+Home".to_string())
    );
    assert_eq!(
        shortcut_hint_for_modifiers(AppShortcut::LastNode, "Cmd", "Option"),
        Some("Option+End".to_string())
    );
}

#[test]
fn view_shortcut_hints_cover_numbered_primary_workspaces() {
    let modifier = command_modifier_label();
    assert_eq!(
        view_shortcut_hint(View::Summary),
        Some(format!("{modifier}+1"))
    );
    assert_eq!(
        view_shortcut_hint(View::Wallets),
        Some(format!("{modifier}+8"))
    );
    assert_eq!(
        view_shortcut_hint(View::Nodes),
        Some(format!("{modifier}+9"))
    );
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
            AppShortcut::ToggleTheme,
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
    assert_eq!(
        shortcut_command_label(AppShortcut::ToggleTheme),
        "Toggle theme"
    );
}
