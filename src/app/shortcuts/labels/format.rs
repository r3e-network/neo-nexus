use super::View;

pub(super) fn command_modifier_label() -> &'static str {
    if cfg!(target_os = "macos") {
        "Cmd"
    } else {
        "Ctrl"
    }
}

pub(super) fn alternate_modifier_label() -> &'static str {
    if cfg!(target_os = "macos") {
        "Option"
    } else {
        "Alt"
    }
}

pub(super) fn numbered_view_key(view: View) -> Option<&'static str> {
    match view {
        View::Summary => Some("1"),
        View::Operations => Some("2"),
        View::Monitor => Some("3"),
        View::Alerts => Some("4"),
        View::Federation => Some("5"),
        View::Settings => Some("6"),
        View::Runtimes => Some("7"),
        View::Wallets => Some("8"),
        View::Nodes => Some("9"),
        View::Roles | View::Snapshots | View::Plugins | View::Config | View::Logs => None,
    }
}

pub(super) fn view_menu_display_label(view: View) -> &'static str {
    match view {
        View::Summary => "Summary",
        View::Operations => "Operations",
        View::Monitor => "Monitor",
        View::Alerts => "Alerts",
        View::Federation => "Federation",
        View::Settings => "Settings",
        View::Runtimes => "Runtimes",
        View::Wallets => "Wallets",
        View::Nodes => "Nodes",
        View::Roles => "Roles",
        View::Snapshots => "Fast Sync",
        View::Plugins => "Plugins",
        View::Config => "Configuration",
        View::Logs => "Runtime Logs",
    }
}

pub(super) fn menu_label(label: &str, shortcut: &str) -> String {
    format!("{label}    {shortcut}")
}

pub(super) fn command_shortcut(modifier: &str, key: &str) -> String {
    format!("{modifier}+{key}")
}

pub(super) fn command_shortcut_context(modifier: &str, key: &str, context: &str) -> String {
    format!("{} {context}", command_shortcut(modifier, key))
}

pub(super) fn alternate_shortcut(modifier: &str, key: &str) -> String {
    format!("{modifier}+{key}")
}
