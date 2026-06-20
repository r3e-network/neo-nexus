use eframe::egui;

use crate::dashboard::DashboardSummary;

use super::super::super::widgets::metric_tile;

pub(super) fn render_overview_metrics(ui: &mut egui::Ui, summary: Option<&DashboardSummary>) {
    ui.horizontal(|ui| {
        metric_tile(
            ui,
            "Health",
            &summary.map_or_else(
                || "-".to_string(),
                |value| format!("{}%", value.health_percent),
            ),
            "running nodes",
        );
        metric_tile(
            ui,
            "Total",
            &summary.map_or_else(|| "-".to_string(), |value| value.total_nodes.to_string()),
            "managed nodes",
        );
        metric_tile(
            ui,
            "Running",
            &summary.map_or_else(|| "-".to_string(), |value| value.running_nodes.to_string()),
            "online",
        );
        metric_tile(
            ui,
            "RPC",
            &summary.map_or_else(
                || "-".to_string(),
                |value| value.rpc_enabled_nodes.to_string(),
            ),
            "plugin ready",
        );
    });
}
