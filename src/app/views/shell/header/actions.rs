use eframe::egui;

use super::{state::NodeActionState, View};
use crate::app::NeoNexusApp;

pub(super) fn render_node_action_buttons(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    let action_state = NodeActionState::from_app(app);

    if ui
        .add_sized([90.0, 30.0], egui::Button::new("New Node"))
        .on_hover_text("Create or import a local Neo runtime definition")
        .clicked()
    {
        app.selected_view = View::Nodes;
    }
    if ui
        .add_sized([74.0, 30.0], egui::Button::new("Reload"))
        .on_hover_text("Reload workspace data and refresh metrics")
        .clicked()
    {
        app.reload_workspace_data();
        app.notice = Some("Workspace reloaded".to_string());
    }
    if ui
        .add_enabled(action_state.can_start, egui::Button::new("Start"))
        .on_hover_text("Start the selected stopped node")
        .clicked()
    {
        app.start_selected_node();
    }
    if ui
        .add_enabled(action_state.can_stop, egui::Button::new("Stop"))
        .on_hover_text("Stop the selected running node")
        .clicked()
    {
        app.stop_selected_node();
    }
    if ui
        .add_enabled(action_state.can_restart, egui::Button::new("Restart"))
        .on_hover_text("Restart the selected running node")
        .clicked()
    {
        app.restart_selected_node();
    }
}
