use eframe::egui;

use super::{state::NodeActionState, View};
use crate::app::{
    shortcuts::{
        labels::{
            node_menu_shortcuts, shortcut_command_label, shortcut_menu_label, view_menu_shortcuts,
            workspace_menu_shortcuts,
        },
        AppShortcut,
    },
    NeoNexusApp,
};

pub(super) fn render_application_menu(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.menu_button("Workspace", |ui| render_workspace_menu(app, ui));
    ui.menu_button("Node", |ui| render_node_menu(app, ui));
    ui.menu_button("View", |ui| render_view_menu(app, ui));
}

fn render_workspace_menu(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    for shortcut in workspace_menu_shortcuts() {
        if matches!(shortcut, AppShortcut::SelectView(View::Summary)) {
            ui.separator();
        }
        render_shortcut_button(app, ui, shortcut, true);
    }
}

fn render_node_menu(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    let action_state = NodeActionState::from_app(app);
    let has_nodes = app.visible_node_count() > 0;
    for shortcut in node_menu_shortcuts() {
        if matches!(shortcut, AppShortcut::PreviousNode) {
            ui.separator();
        }
        if matches!(shortcut, AppShortcut::StartSelectedNode) {
            ui.separator();
        }
        render_shortcut_button(
            app,
            ui,
            shortcut,
            node_shortcut_enabled(shortcut, &action_state, has_nodes),
        );
    }
}

fn render_view_menu(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    for shortcut in view_menu_shortcuts() {
        if matches!(shortcut, AppShortcut::SelectView(View::Summary)) {
            ui.separator();
        }
        render_view_shortcut(app, ui, shortcut);
    }
}

fn render_shortcut_button(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    shortcut: AppShortcut,
    enabled: bool,
) {
    if ui
        .add_enabled(enabled, egui::Button::new(shortcut_command_label(shortcut)))
        .on_hover_text(shortcut_menu_label(shortcut))
        .clicked()
    {
        app.apply_application_shortcut(shortcut);
    }
}

fn render_view_shortcut(app: &mut NeoNexusApp, ui: &mut egui::Ui, shortcut: AppShortcut) {
    let selected = matches!(shortcut, AppShortcut::SelectView(view) if app.selected_view == view);
    if ui
        .selectable_label(selected, shortcut_command_label(shortcut))
        .on_hover_text(shortcut_menu_label(shortcut))
        .clicked()
    {
        app.apply_application_shortcut(shortcut);
    }
}

fn node_shortcut_enabled(
    shortcut: AppShortcut,
    action_state: &NodeActionState,
    has_nodes: bool,
) -> bool {
    match shortcut {
        AppShortcut::NewNode => true,
        AppShortcut::PreviousNode
        | AppShortcut::NextNode
        | AppShortcut::PreviousNodePage
        | AppShortcut::NextNodePage
        | AppShortcut::FirstNode
        | AppShortcut::LastNode => has_nodes,
        AppShortcut::StartSelectedNode => action_state.can_start,
        AppShortcut::StopSelectedNode => action_state.can_stop,
        AppShortcut::RestartSelectedNode => action_state.can_restart,
        AppShortcut::ReloadWorkspace
        | AppShortcut::ToggleSelectedNode
        | AppShortcut::NextView
        | AppShortcut::PreviousView
        | AppShortcut::SelectView(_) => false,
    }
}
