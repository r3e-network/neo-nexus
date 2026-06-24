use eframe::egui;

use super::state::NodeActionState;
use crate::app::{
    shortcuts::{
        labels::{primary_action_shortcuts, shortcut_hint, shortcut_toolbar_label},
        AppShortcut,
    },
    NeoNexusApp,
};

pub(super) fn render_node_action_buttons(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    let action_state = NodeActionState::from_app(app);

    for shortcut in primary_action_shortcuts() {
        if render_primary_action_button(ui, shortcut, &action_state) {
            app.apply_application_shortcut(shortcut);
        }
    }
}

fn render_primary_action_button(
    ui: &mut egui::Ui,
    shortcut: AppShortcut,
    action_state: &NodeActionState,
) -> bool {
    ui.add_enabled_ui(primary_action_enabled(shortcut, action_state), |ui| {
        ui.add_sized(
            [primary_action_width(shortcut), 28.0],
            egui::Button::new(shortcut_toolbar_label(shortcut)),
        )
    })
    .inner
    .on_hover_text(primary_action_hint(shortcut))
    .clicked()
}

fn primary_action_enabled(shortcut: AppShortcut, action_state: &NodeActionState) -> bool {
    match shortcut {
        AppShortcut::NewNode | AppShortcut::ReloadWorkspace => true,
        AppShortcut::StartSelectedNode => action_state.can_start,
        AppShortcut::StopSelectedNode => action_state.can_stop,
        AppShortcut::RestartSelectedNode => action_state.can_restart,
        AppShortcut::ToggleSelectedNode
        | AppShortcut::PreviousNode
        | AppShortcut::NextNode
        | AppShortcut::PreviousNodePage
        | AppShortcut::NextNodePage
        | AppShortcut::FirstNode
        | AppShortcut::LastNode
        | AppShortcut::NextView
        | AppShortcut::PreviousView
        | AppShortcut::ToggleTheme
        | AppShortcut::SelectView(_) => false,
    }
}

fn primary_action_width(shortcut: AppShortcut) -> f32 {
    match shortcut {
        AppShortcut::NewNode => 90.0,
        AppShortcut::ReloadWorkspace
        | AppShortcut::StartSelectedNode
        | AppShortcut::StopSelectedNode
        | AppShortcut::RestartSelectedNode => 74.0,
        _ => 90.0,
    }
}

fn primary_action_hint(shortcut: AppShortcut) -> String {
    let action = match shortcut {
        AppShortcut::NewNode => "Create or import a local Neo runtime definition",
        AppShortcut::ReloadWorkspace => "Reload workspace data and refresh metrics",
        AppShortcut::StartSelectedNode => "Start the selected stopped node",
        AppShortcut::StopSelectedNode => "Stop the selected running node",
        AppShortcut::RestartSelectedNode => "Restart the selected running node",
        _ => "Apply command",
    };
    shortcut_hint(shortcut).map_or_else(|| action.to_string(), |hint| format!("{action} ({hint})"))
}
