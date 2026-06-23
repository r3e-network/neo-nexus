use eframe::egui;

use crate::app::{
    domain::{RpcHealthRecord, RpcHealthStatus},
    text::truncate_middle,
    widgets::fact,
    NeoNexusApp,
};

pub(super) fn render_rpc_health(app: &NeoNexusApp, ui: &mut egui::Ui, node_id: &str) {
    match app.repository.list_rpc_health(node_id, 5) {
        Ok(history) => {
            if let Some(health) = history.first() {
                fact(ui, "RPC Health", health.status.label());
                fact(
                    ui,
                    "RPC Height",
                    &health
                        .block_count
                        .map_or_else(|| "-".to_string(), |height| height.to_string()),
                );
                fact(ui, "RPC Checked", &health.checked_at_unix.to_string());
                fact(ui, "RPC Recent", &rpc_health_trend(&history));
                fact(ui, "RPC Message", &truncate_middle(&health.message, 40));
            } else {
                fact(ui, "RPC Health", "unchecked");
            }
        }
        Err(error) => {
            fact(ui, "RPC Health", &truncate_middle(&error.to_string(), 40));
        }
    }
}

fn rpc_health_trend(history: &[RpcHealthRecord]) -> String {
    let healthy = history
        .iter()
        .filter(|record| record.status == RpcHealthStatus::Healthy)
        .count();
    let degraded = history
        .iter()
        .filter(|record| record.status == RpcHealthStatus::Degraded)
        .count();
    let unreachable = history
        .iter()
        .filter(|record| record.status == RpcHealthStatus::Unreachable)
        .count();

    format!(
        "{} samples | H/D/U {healthy}/{degraded}/{unreachable}",
        history.len()
    )
}
