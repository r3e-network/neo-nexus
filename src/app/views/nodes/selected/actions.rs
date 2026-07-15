use eframe::egui;

use crate::app::{
    domain::NodeStatus,
    theme,
    widgets::{confirm_bar, toolbar, ToolbarAction},
    NeoNexusApp,
};

pub(super) fn render_action_bar(app: &mut NeoNexusApp, ui: &mut egui::Ui, status: NodeStatus) {
    ui.add_space(theme::MD);
    ui.label(theme::label_caption("Tools"));
    ui.add_space(theme::XS);
    let actions = [
        ToolbarAction::secondary("load", "Load Into Draft")
            .hint("Copy this definition into the Studio draft"),
        ToolbarAction::secondary("probe", "Probe Binary").hint("Inspect the node binary path"),
        ToolbarAction::secondary("smoke", "Smoke Runtime")
            .hint("Run a short runtime smoke probe"),
        ToolbarAction::secondary("rpc", "RPC Health").hint("Probe the node RPC endpoint"),
        ToolbarAction::secondary("ports", "Fix Ports")
            .enabled(!status.is_active())
            .hint("Assign free ports to this stopped node"),
        ToolbarAction::secondary("delete", "Delete")
            .enabled(!status.is_running())
            .hint("Delete this node definition"),
    ];
    if let Some(id) = toolbar(ui, &actions) {
        match id {
            "load" => app.load_selected_node_into_draft(),
            "probe" => app.probe_selected_binary(),
            "smoke" => app.smoke_selected_runtime(),
            "rpc" => app.check_selected_rpc_health(),
            "ports" => app.assign_available_ports_to_selected_node(),
            "delete" => app.request_delete_selected_node(),
            _ => {}
        }
    }
}

pub(super) fn render_delete_confirmation(app: &mut NeoNexusApp, ui: &mut egui::Ui, node_id: &str) {
    if app.fleet.pending_delete_node.as_deref() != Some(node_id) {
        return;
    }

    match confirm_bar(
        ui,
        "Delete this node?",
        "Removes the definition and plugin state. Running nodes must be stopped first.",
        "Confirm Delete",
        "Cancel",
        true,
    ) {
        Some(true) => app.confirm_delete_node(),
        Some(false) => app.cancel_delete_node(),
        None => {}
    }
}
