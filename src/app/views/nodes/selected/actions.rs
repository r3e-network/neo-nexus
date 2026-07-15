use eframe::egui;

use crate::app::{
    domain::NodeStatus,
    theme,
    widgets::{callout, primary_button, secondary_button, secondary_button_enabled, CalloutKind},
    NeoNexusApp,
};

pub(super) fn render_action_bar(app: &mut NeoNexusApp, ui: &mut egui::Ui, status: NodeStatus) {
    ui.add_space(theme::MD);
    ui.label(theme::label_caption("Tools"));
    ui.add_space(theme::XS);
    ui.horizontal_wrapped(|ui| {
        if secondary_button(ui, "Load Into Draft")
            .on_hover_text("Copy this definition into the Studio draft")
            .clicked()
        {
            app.load_selected_node_into_draft();
        }
        if secondary_button(ui, "Probe Binary")
            .on_hover_text("Inspect the node binary path")
            .clicked()
        {
            app.probe_selected_binary();
        }
        if secondary_button(ui, "Smoke Runtime")
            .on_hover_text("Run a short runtime smoke probe")
            .clicked()
        {
            app.smoke_selected_runtime();
        }
        if secondary_button(ui, "RPC Health")
            .on_hover_text("Probe the node RPC endpoint")
            .clicked()
        {
            app.check_selected_rpc_health();
        }
        if secondary_button_enabled(ui, "Fix Ports", !status.is_active())
            .on_hover_text("Assign free ports to this stopped node")
            .clicked()
        {
            app.assign_available_ports_to_selected_node();
        }
        if secondary_button_enabled(ui, "Delete", !status.is_running())
            .on_hover_text("Delete this node definition")
            .clicked()
        {
            app.request_delete_selected_node();
        }
    });
}

pub(super) fn render_delete_confirmation(app: &mut NeoNexusApp, ui: &mut egui::Ui, node_id: &str) {
    if app.fleet.pending_delete_node.as_deref() != Some(node_id) {
        return;
    }

    ui.add_space(theme::MD);
    callout(
        ui,
        CalloutKind::Danger,
        "Delete this node?",
        "Removes the definition and plugin state. Running nodes must be stopped first.",
    );
    ui.add_space(theme::SM);
    ui.horizontal(|ui| {
        if primary_button(ui, "Confirm Delete").clicked() {
            app.confirm_delete_node();
        }
        if secondary_button(ui, "Cancel").clicked() {
            app.cancel_delete_node();
        }
    });
}
