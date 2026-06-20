use eframe::egui;

use super::{state::NodeActionState, View};
use crate::app::NeoNexusApp;

pub(super) fn render_application_menu(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.menu_button("Workspace", |ui| render_workspace_menu(app, ui));
    ui.menu_button("Node", |ui| render_node_menu(app, ui));
    ui.menu_button("View", |ui| render_view_menu(app, ui));
}

fn render_workspace_menu(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    if ui.button("Reload").clicked() {
        app.reload_workspace_data();
        app.notice = Some("Workspace reloaded".to_string());
    }
    if ui.button("Summary").clicked() {
        app.selected_view = View::Summary;
    }
}

fn render_node_menu(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    let action_state = NodeActionState::from_app(app);
    if ui.button("New node").clicked() {
        app.selected_view = View::Nodes;
    }
    ui.separator();
    if ui
        .add_enabled(action_state.can_start, egui::Button::new("Start selected"))
        .clicked()
    {
        app.start_selected_node();
    }
    if ui
        .add_enabled(action_state.can_stop, egui::Button::new("Stop selected"))
        .clicked()
    {
        app.stop_selected_node();
    }
    if ui
        .add_enabled(
            action_state.can_restart,
            egui::Button::new("Restart selected"),
        )
        .clicked()
    {
        app.restart_selected_node();
    }
}

fn render_view_menu(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    for view in View::ALL {
        if ui
            .selectable_label(app.selected_view == view, view.label())
            .clicked()
        {
            app.selected_view = view;
        }
    }
}
