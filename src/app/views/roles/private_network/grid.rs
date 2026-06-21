use eframe::egui;

use crate::app::domain::{PrivateNetworkNodePlan, PrivateNetworkPlan};

use super::super::super::super::text::truncate_middle;

const PRIVATE_PLAN_ROWS: usize = 7;
const PRIVATE_PLAN_COLUMNS: usize = 7;

pub(super) fn render_plan_grid(ui: &mut egui::Ui, plan: &PrivateNetworkPlan) {
    egui::Grid::new("private_network_plan")
        .striped(true)
        .min_col_width(74.0)
        .show(ui, |ui| {
            render_header(ui);
            for row in 0..PRIVATE_PLAN_ROWS {
                render_row(ui, plan.nodes.get(row));
                ui.end_row();
            }
        });
}

fn render_header(ui: &mut egui::Ui) {
    ui.strong("Name");
    ui.strong("Runtime");
    ui.strong("Role");
    ui.strong("RPC");
    ui.strong("P2P");
    ui.strong("WS");
    ui.strong("Storage");
    ui.end_row();
}

fn render_row(ui: &mut egui::Ui, node: Option<&PrivateNetworkNodePlan>) {
    if let Some(node) = node {
        ui.label(truncate_middle(&node.name, 26));
        ui.label(node.node_type.to_string());
        ui.label(node.role.label());
        ui.label(node.rpc_port.to_string());
        ui.label(node.p2p_port.to_string());
        ui.label(
            node.ws_port
                .map_or_else(|| "-".to_string(), |port| port.to_string()),
        );
        ui.label(node.storage_engine.to_string());
    } else {
        for _ in 0..PRIVATE_PLAN_COLUMNS {
            ui.label(" ");
        }
    }
}
