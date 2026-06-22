use eframe::egui;

use crate::dashboard::DashboardSummary;

use super::super::super::widgets::metric_row;

pub(super) fn render_overview_metrics(ui: &mut egui::Ui, summary: Option<&DashboardSummary>) {
    let health = summary.map_or_else(
        || "-".to_string(),
        |value| format!("{}%", value.health_percent),
    );
    let total = summary.map_or_else(|| "-".to_string(), |value| value.total_nodes.to_string());
    let running = summary.map_or_else(|| "-".to_string(), |value| value.running_nodes.to_string());
    let rpc = summary.map_or_else(
        || "-".to_string(),
        |value| value.rpc_enabled_nodes.to_string(),
    );

    metric_row(
        ui,
        &[
            ("Health", &health, "running nodes"),
            ("Total", &total, "managed nodes"),
            ("Running", &running, "online"),
            ("RPC", &rpc, "plugin ready"),
        ],
    );
}
