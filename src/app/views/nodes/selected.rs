mod actions;
mod health;
mod readiness;
mod summary;

use eframe::egui;

use super::super::super::{widgets::empty_state, NeoNexusApp};

pub(super) fn render_selected_node_editor(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    let Some(node) = app.selected_node().cloned() else {
        empty_state(ui, "No selection", "Select a node from Inventory.");
        return;
    };

    summary::render_node_summary(app, ui, &node);
    readiness::render_readiness(app, ui, &node);
    health::render_rpc_health(app, ui, &node.id);
    actions::render_action_bar(app, ui, node.status);
    actions::render_delete_confirmation(app, ui, &node.id);
}
