use eframe::egui;

use crate::app::{
    domain::NodeStatus,
    theme::{self, status_color},
    widgets, NeoNexusApp,
};

pub(super) fn render_action_bar(app: &mut NeoNexusApp, ui: &mut egui::Ui, status: NodeStatus) {
    ui.add_space(theme::MD);
    ui.horizontal(|ui| {
        if widgets::secondary_button(ui, "Load Into Draft").clicked() {
            app.load_selected_node_into_draft();
        }
        if widgets::secondary_button(ui, "Probe Binary").clicked() {
            app.probe_selected_binary();
        }
        if widgets::secondary_button(ui, "Smoke Runtime").clicked() {
            app.smoke_selected_runtime();
        }
        if widgets::secondary_button(ui, "RPC Health").clicked() {
            app.check_selected_rpc_health();
        }
        if widgets::secondary_button_enabled(ui, "Fix Ports", !status.is_active()).clicked() {
            app.assign_available_ports_to_selected_node();
        }
        if widgets::secondary_button_enabled(ui, "Delete", !status.is_running()).clicked() {
            app.request_delete_selected_node();
        }
    });
}

pub(super) fn render_delete_confirmation(app: &mut NeoNexusApp, ui: &mut egui::Ui, node_id: &str) {
    if app.fleet.pending_delete_node.as_deref() != Some(node_id) {
        return;
    }

    ui.separator();
    ui.label(
        egui::RichText::new("Delete this node definition and plugin state?")
            .color(status_color(NodeStatus::Error))
            .strong(),
    );
    ui.horizontal(|ui| {
        if widgets::primary_button(ui, "Confirm Delete").clicked() {
            app.confirm_delete_node();
        }
        if widgets::secondary_button(ui, "Cancel").clicked() {
            app.cancel_delete_node();
        }
    });
}
