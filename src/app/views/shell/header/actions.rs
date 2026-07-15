use eframe::egui;

use super::state::NodeActionState;
use crate::app::{
    shortcuts::{
        labels::{primary_action_shortcuts, shortcut_hint, shortcut_toolbar_label},
        AppShortcut,
    },
    theme,
    view::View,
    widgets::{primary_button, secondary_button_enabled},
    NeoNexusApp,
};

pub(super) fn render_node_action_buttons(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    let action_state = NodeActionState::from_app(app);
    let show_lifecycle = shows_lifecycle_actions(app.session.selected_view);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = theme::SM;
        for shortcut in primary_action_shortcuts() {
            if !show_lifecycle && is_lifecycle_shortcut(shortcut) {
                continue;
            }
            if render_action_button(ui, shortcut, &action_state) {
                app.apply_application_shortcut(shortcut);
            }
        }
    });
}

fn shows_lifecycle_actions(view: View) -> bool {
    matches!(
        view.primary_nav(),
        View::Summary | View::Nodes | View::Operations
    )
}

fn is_lifecycle_shortcut(shortcut: AppShortcut) -> bool {
    matches!(
        shortcut,
        AppShortcut::StartSelectedNode
            | AppShortcut::StopSelectedNode
            | AppShortcut::RestartSelectedNode
    )
}

fn render_action_button(
    ui: &mut egui::Ui,
    shortcut: AppShortcut,
    action_state: &NodeActionState,
) -> bool {
    let enabled = primary_action_enabled(shortcut, action_state);
    let label = shortcut_toolbar_label(shortcut);
    let response = match shortcut {
        // Dominant create action always uses the accent fill.
        AppShortcut::NewNode => primary_button(ui, label),
        AppShortcut::StartSelectedNode if enabled => primary_button(ui, label),
        _ => secondary_button_enabled(ui, label, enabled),
    };
    response
        .on_hover_text(primary_action_hint(shortcut))
        .clicked()
}

fn primary_action_enabled(shortcut: AppShortcut, action_state: &NodeActionState) -> bool {
    match shortcut {
        AppShortcut::NewNode | AppShortcut::ReloadWorkspace => true,
        AppShortcut::StartSelectedNode => action_state.can_start,
        AppShortcut::StopSelectedNode => action_state.can_stop,
        AppShortcut::RestartSelectedNode => action_state.can_restart,
        _ => false,
    }
}

fn primary_action_hint(shortcut: AppShortcut) -> String {
    let action = match shortcut {
        AppShortcut::NewNode => "Create a local Neo runtime definition",
        AppShortcut::ReloadWorkspace => "Reload workspace data and refresh metrics",
        AppShortcut::StartSelectedNode => "Start the selected stopped node",
        AppShortcut::StopSelectedNode => "Stop the selected running node",
        AppShortcut::RestartSelectedNode => "Restart the selected running node",
        _ => "Apply command",
    };
    shortcut_hint(shortcut).map_or_else(|| action.to_string(), |hint| format!("{action} ({hint})"))
}
