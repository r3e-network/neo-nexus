use super::super::{AppShortcut, View};

pub(in crate::app) fn shortcut_command_label(shortcut: AppShortcut) -> &'static str {
    match shortcut {
        AppShortcut::ReloadWorkspace => "Reload",
        AppShortcut::NewNode => "New node",
        AppShortcut::StartSelectedNode => "Start selected",
        AppShortcut::StopSelectedNode => "Stop selected",
        AppShortcut::RestartSelectedNode => "Restart selected",
        AppShortcut::ToggleSelectedNode => "Start / stop selected",
        AppShortcut::PreviousNode => "Previous node",
        AppShortcut::NextNode => "Next node",
        AppShortcut::PreviousNodePage => "Previous node page",
        AppShortcut::NextNodePage => "Next node page",
        AppShortcut::FirstNode => "First node",
        AppShortcut::LastNode => "Last node",
        AppShortcut::NextView => "Next view",
        AppShortcut::PreviousView => "Previous view",
        AppShortcut::ToggleTheme => "Toggle theme",
        AppShortcut::SelectView(view) => view.label(),
    }
}

pub(in crate::app) fn shortcut_toolbar_label(shortcut: AppShortcut) -> &'static str {
    match shortcut {
        AppShortcut::NewNode => "New Node",
        AppShortcut::ReloadWorkspace => "Reload",
        AppShortcut::StartSelectedNode => "Start",
        AppShortcut::StopSelectedNode => "Stop",
        AppShortcut::RestartSelectedNode => "Restart",
        _ => shortcut_command_label(shortcut),
    }
}

pub(in crate::app) fn primary_action_shortcuts() -> [AppShortcut; 5] {
    [
        AppShortcut::NewNode,
        AppShortcut::ReloadWorkspace,
        AppShortcut::StartSelectedNode,
        AppShortcut::StopSelectedNode,
        AppShortcut::RestartSelectedNode,
    ]
}

pub(in crate::app) fn workspace_menu_shortcuts() -> [AppShortcut; 4] {
    [
        AppShortcut::ReloadWorkspace,
        AppShortcut::SelectView(View::Summary),
        AppShortcut::SelectView(View::Settings),
        AppShortcut::ToggleTheme,
    ]
}

pub(in crate::app) fn node_menu_shortcuts() -> [AppShortcut; 10] {
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
}

pub(in crate::app) fn view_menu_shortcuts() -> Vec<AppShortcut> {
    // v3: menu lists the six primary destinations only; legacy tools are reached
    // through in-page sections/tabs rather than top-level SelectView entries.
    let mut shortcuts = Vec::with_capacity(View::PRIMARY.len() + 2);
    shortcuts.push(AppShortcut::PreviousView);
    shortcuts.push(AppShortcut::NextView);
    shortcuts.extend(View::PRIMARY.into_iter().map(AppShortcut::SelectView));
    shortcuts
}
