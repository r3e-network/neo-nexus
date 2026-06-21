use eframe::egui;

use crate::app::{domain::NodeStatus, theme::status_color, NeoNexusApp};

pub(super) fn render_action_bar(app: &mut NeoNexusApp, ui: &mut egui::Ui, status: NodeStatus) {
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        if ui.button("Load Into Draft").clicked() {
            app.load_selected_node_into_draft();
        }
        if ui.button("Probe Binary").clicked() {
            app.probe_selected_binary();
        }
        if ui.button("Smoke Runtime").clicked() {
            app.smoke_selected_runtime();
        }
        if ui.button("RPC Health").clicked() {
            app.check_selected_rpc_health();
        }
        if ui
            .add_enabled(!status.is_active(), egui::Button::new("Fix Ports"))
            .clicked()
        {
            app.assign_available_ports_to_selected_node();
        }
        if ui
            .add_enabled(!status.is_running(), egui::Button::new("Delete"))
            .clicked()
        {
            app.request_delete_selected_node();
        }
    });
}

pub(super) fn render_delete_confirmation(app: &mut NeoNexusApp, ui: &mut egui::Ui, node_id: &str) {
    if app.pending_delete_node.as_deref() != Some(node_id) {
        return;
    }

    ui.separator();
    ui.label(
        egui::RichText::new("Delete this node definition and plugin state?")
            .color(status_color(NodeStatus::Error))
            .strong(),
    );
    ui.horizontal(|ui| {
        if ui.button("Confirm Delete").clicked() {
            app.confirm_delete_node();
        }
        if ui.button("Cancel").clicked() {
            app.cancel_delete_node();
        }
    });
}
