use eframe::egui;

use crate::{app::theme::muted_text, types::NodeConfig};

pub(in crate::app) fn render_node_fact_sheet(ui: &mut egui::Ui, node: &NodeConfig) {
    fact(ui, "Name", &node.name);
    fact(ui, "Type", &node.node_type.to_string());
    fact(ui, "Network", &node.network.to_string());
    fact(ui, "Version", &node.runtime_version);
    fact(ui, "Storage", &node.storage_engine.to_string());
    fact(ui, "RPC", &node.rpc_port.to_string());
    fact(ui, "P2P", &node.p2p_port.to_string());
    fact(
        ui,
        "WebSocket",
        &node
            .ws_port
            .map_or_else(|| "-".to_string(), |port| port.to_string()),
    );
}

pub(in crate::app) fn fact(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.set_min_height(22.0);
        ui.label(egui::RichText::new(label).color(muted_text()));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(value);
        });
    });
}
