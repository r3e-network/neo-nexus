use super::{AppShortcut, View};

mod format;
mod lists;

pub(in crate::app) use self::lists::{
    node_menu_shortcuts, primary_action_shortcuts, shortcut_command_label, shortcut_toolbar_label,
    view_menu_shortcuts, workspace_menu_shortcuts,
};

use self::format::{
    alternate_modifier_label, alternate_shortcut, command_shortcut, command_shortcut_context,
    menu_label, numbered_view_key, view_menu_display_label,
};

pub(in crate::app) fn command_modifier_label() -> &'static str {
    format::command_modifier_label()
}

pub(in crate::app) fn alternate_navigation_modifier_label() -> &'static str {
    alternate_modifier_label()
}

pub(in crate::app) fn shortcut_menu_label(shortcut: AppShortcut) -> String {
    shortcut_menu_label_for_modifiers(
        shortcut,
        command_modifier_label(),
        alternate_navigation_modifier_label(),
    )
}

#[cfg(test)]
pub(in crate::app) fn shortcut_menu_label_for_modifier(
    shortcut: AppShortcut,
    modifier: &str,
) -> String {
    shortcut_menu_label_for_modifiers(shortcut, modifier, "Alt")
}

pub(in crate::app) fn shortcut_menu_label_for_modifiers(
    shortcut: AppShortcut,
    command_modifier: &str,
    alternate_modifier: &str,
) -> String {
    match shortcut {
        AppShortcut::ReloadWorkspace => menu_label("Reload", &format!("{command_modifier}+R / F5")),
        AppShortcut::NewNode => menu_label("New node", &command_shortcut(command_modifier, "N")),
        AppShortcut::StartSelectedNode => menu_label(
            "Start selected",
            &command_shortcut_context(command_modifier, "Enter", "when stopped"),
        ),
        AppShortcut::StopSelectedNode => menu_label(
            "Stop selected",
            &command_shortcut_context(command_modifier, "Enter", "when running"),
        ),
        AppShortcut::RestartSelectedNode => menu_label(
            "Restart selected",
            &command_shortcut(command_modifier, "Shift+Enter"),
        ),
        AppShortcut::ToggleSelectedNode => menu_label(
            "Start / stop selected",
            &command_shortcut(command_modifier, "Enter"),
        ),
        AppShortcut::PreviousNode => menu_label(
            "Previous node",
            &alternate_shortcut(alternate_modifier, "Up"),
        ),
        AppShortcut::NextNode => {
            menu_label("Next node", &alternate_shortcut(alternate_modifier, "Down"))
        }
        AppShortcut::PreviousNodePage => menu_label(
            "Previous node page",
            &alternate_shortcut(alternate_modifier, "PageUp"),
        ),
        AppShortcut::NextNodePage => menu_label(
            "Next node page",
            &alternate_shortcut(alternate_modifier, "PageDown"),
        ),
        AppShortcut::FirstNode => menu_label(
            "First node",
            &alternate_shortcut(alternate_modifier, "Home"),
        ),
        AppShortcut::LastNode => {
            menu_label("Last node", &alternate_shortcut(alternate_modifier, "End"))
        }
        AppShortcut::NextView => menu_label("Next view", &command_shortcut(command_modifier, "]")),
        AppShortcut::PreviousView => {
            menu_label("Previous view", &command_shortcut(command_modifier, "["))
        }
        AppShortcut::ToggleTheme => {
            menu_label("Toggle theme", &command_shortcut(command_modifier, "D"))
        }
        AppShortcut::SelectView(view) => view_menu_label_for_modifier(view, command_modifier),
    }
}

pub(in crate::app) fn view_menu_label_for_modifier(view: View, modifier: &str) -> String {
    let label = view_menu_display_label(view);
    match numbered_view_key(view) {
        Some(key) => menu_label(label, &command_shortcut(modifier, key)),
        None => label.to_string(),
    }
}

pub(in crate::app) fn shortcut_hint(shortcut: AppShortcut) -> Option<String> {
    shortcut_hint_for_modifiers(
        shortcut,
        command_modifier_label(),
        alternate_navigation_modifier_label(),
    )
}

#[cfg(test)]
pub(in crate::app) fn shortcut_hint_for_modifier(
    shortcut: AppShortcut,
    modifier: &str,
) -> Option<String> {
    shortcut_hint_for_modifiers(shortcut, modifier, "Alt")
}

pub(in crate::app) fn shortcut_hint_for_modifiers(
    shortcut: AppShortcut,
    command_modifier: &str,
    alternate_modifier: &str,
) -> Option<String> {
    match shortcut {
        AppShortcut::ReloadWorkspace => Some(format!("{command_modifier}+R or F5")),
        AppShortcut::NewNode => Some(command_shortcut(command_modifier, "N")),
        AppShortcut::StartSelectedNode => Some(command_shortcut_context(
            command_modifier,
            "Enter",
            "when stopped",
        )),
        AppShortcut::StopSelectedNode => Some(command_shortcut_context(
            command_modifier,
            "Enter",
            "when running",
        )),
        AppShortcut::RestartSelectedNode => Some(command_shortcut(command_modifier, "Shift+Enter")),
        AppShortcut::ToggleSelectedNode => Some(command_shortcut(command_modifier, "Enter")),
        AppShortcut::PreviousNode => Some(alternate_shortcut(alternate_modifier, "Up")),
        AppShortcut::NextNode => Some(alternate_shortcut(alternate_modifier, "Down")),
        AppShortcut::PreviousNodePage => Some(alternate_shortcut(alternate_modifier, "PageUp")),
        AppShortcut::NextNodePage => Some(alternate_shortcut(alternate_modifier, "PageDown")),
        AppShortcut::FirstNode => Some(alternate_shortcut(alternate_modifier, "Home")),
        AppShortcut::LastNode => Some(alternate_shortcut(alternate_modifier, "End")),
        AppShortcut::NextView => Some(command_shortcut(command_modifier, "]")),
        AppShortcut::PreviousView => Some(command_shortcut(command_modifier, "[")),
        AppShortcut::ToggleTheme => Some(command_shortcut(command_modifier, "D")),
        AppShortcut::SelectView(view) => view_shortcut_hint_for_modifier(view, command_modifier),
    }
}

#[cfg(test)]
pub(in crate::app) fn view_shortcut_hint(view: View) -> Option<String> {
    view_shortcut_hint_for_modifier(view, command_modifier_label())
}

pub(in crate::app) fn view_shortcut_hint_for_modifier(
    view: View,
    modifier: &str,
) -> Option<String> {
    numbered_view_key(view).map(|key| command_shortcut(modifier, key))
}
