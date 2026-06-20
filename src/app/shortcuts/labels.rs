use super::{AppShortcut, View};

pub(in crate::app) fn shortcut_menu_label(shortcut: AppShortcut) -> &'static str {
    match shortcut {
        AppShortcut::ReloadWorkspace => "Reload    Cmd+R / F5",
        AppShortcut::NewNode => "New node    Cmd+N",
        AppShortcut::StartSelectedNode => "Start selected    Cmd+Enter when stopped",
        AppShortcut::StopSelectedNode => "Stop selected    Cmd+Enter when running",
        AppShortcut::RestartSelectedNode => "Restart selected    Cmd+Shift+Enter",
        AppShortcut::ToggleSelectedNode => "Start / stop selected    Cmd+Enter",
        AppShortcut::PreviousNode => "Previous node    Alt+Up",
        AppShortcut::NextNode => "Next node    Alt+Down",
        AppShortcut::PreviousNodePage => "Previous node page    Alt+PageUp",
        AppShortcut::NextNodePage => "Next node page    Alt+PageDown",
        AppShortcut::FirstNode => "First node    Alt+Home",
        AppShortcut::LastNode => "Last node    Alt+End",
        AppShortcut::NextView => "Next view    Cmd+]",
        AppShortcut::PreviousView => "Previous view    Cmd+[",
        AppShortcut::SelectView(view) => view_menu_label(view),
    }
}

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

pub(in crate::app) fn workspace_menu_shortcuts() -> [AppShortcut; 3] {
    [
        AppShortcut::ReloadWorkspace,
        AppShortcut::SelectView(View::Summary),
        AppShortcut::SelectView(View::Settings),
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
    let mut shortcuts = Vec::with_capacity(View::ALL.len() + 2);
    shortcuts.push(AppShortcut::PreviousView);
    shortcuts.push(AppShortcut::NextView);
    shortcuts.extend(View::ALL.into_iter().map(AppShortcut::SelectView));
    shortcuts
}

pub(in crate::app) fn view_menu_label(view: View) -> &'static str {
    match view {
        View::Summary => "Summary    Cmd+1",
        View::Operations => "Operations    Cmd+2",
        View::Monitor => "Monitor    Cmd+3",
        View::Alerts => "Alerts    Cmd+4",
        View::Federation => "Federation    Cmd+5",
        View::Settings => "Settings    Cmd+6",
        View::Runtimes => "Runtimes    Cmd+7",
        View::Wallets => "Wallets    Cmd+8",
        View::Nodes => "Nodes    Cmd+9",
        View::Roles => "Roles",
        View::Snapshots => "Fast Sync",
        View::Plugins => "Plugins",
        View::Config => "Configuration",
        View::Logs => "Runtime Logs",
    }
}

pub(in crate::app) fn shortcut_hint(shortcut: AppShortcut) -> Option<&'static str> {
    match shortcut {
        AppShortcut::ReloadWorkspace => Some("Cmd+R or F5"),
        AppShortcut::NewNode => Some("Cmd+N"),
        AppShortcut::StartSelectedNode => Some("Cmd+Enter when stopped"),
        AppShortcut::StopSelectedNode => Some("Cmd+Enter when running"),
        AppShortcut::RestartSelectedNode => Some("Cmd+Shift+Enter"),
        AppShortcut::ToggleSelectedNode => Some("Cmd+Enter"),
        AppShortcut::PreviousNode => Some("Alt+Up"),
        AppShortcut::NextNode => Some("Alt+Down"),
        AppShortcut::PreviousNodePage => Some("Alt+PageUp"),
        AppShortcut::NextNodePage => Some("Alt+PageDown"),
        AppShortcut::FirstNode => Some("Alt+Home"),
        AppShortcut::LastNode => Some("Alt+End"),
        AppShortcut::NextView => Some("Cmd+]"),
        AppShortcut::PreviousView => Some("Cmd+["),
        AppShortcut::SelectView(view) => view_shortcut_hint(view),
    }
}

pub(in crate::app) fn view_shortcut_hint(view: View) -> Option<&'static str> {
    match view {
        View::Summary => Some("Cmd+1"),
        View::Operations => Some("Cmd+2"),
        View::Monitor => Some("Cmd+3"),
        View::Alerts => Some("Cmd+4"),
        View::Federation => Some("Cmd+5"),
        View::Settings => Some("Cmd+6"),
        View::Runtimes => Some("Cmd+7"),
        View::Wallets => Some("Cmd+8"),
        View::Nodes => Some("Cmd+9"),
        View::Roles | View::Snapshots | View::Plugins | View::Config | View::Logs => None,
    }
}
