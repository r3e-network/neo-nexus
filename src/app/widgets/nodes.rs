use eframe::egui;

use crate::app::{
    domain::NodeConfig,
    theme::{self, danger},
    widgets::status_badge,
};

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
            .map_or_else(|| "—".to_string(), |port| port.to_string()),
    );
}

/// Label-left / value-right fact row used in inspectors and detail sheets.
pub(in crate::app) fn fact(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.set_min_height(24.0);
        ui.label(theme::muted_body(label));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(theme::body(value));
        });
    });
}

/// Fact row whose value is drawn in the semantic danger colour.
pub(in crate::app) fn fact_error(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.set_min_height(24.0);
        ui.label(theme::muted_body(label));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(theme::body(value).color(danger()).strong());
        });
    });
}

/// Compact node identity strip: name + status badge (inspectors, selection).
#[allow(dead_code)]
pub(in crate::app) fn node_identity_header(
    ui: &mut egui::Ui,
    name: &str,
    status: crate::app::domain::NodeStatus,
) {
    ui.horizontal(|ui| {
        ui.label(theme::section_title(name));
        ui.add_space(theme::SM);
        status_badge(ui, status);
    });
}
