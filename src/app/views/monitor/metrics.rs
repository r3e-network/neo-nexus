use eframe::egui;

use crate::app::domain::format_bytes;

use super::super::super::{widgets::metric_tile, NeoNexusApp};

pub(super) fn render_monitor_metrics(app: &NeoNexusApp, ui: &mut egui::Ui) {
    let system = &app.metrics_snapshot.system;
    ui.horizontal(|ui| {
        metric_tile(
            ui,
            "CPU",
            &format!("{:.0}%", system.cpu_usage_percent),
            system.cpu_pressure().label(),
        );
        metric_tile(
            ui,
            "Memory",
            &format!("{:.0}%", system.memory_usage_percent),
            system.memory_pressure().label(),
        );
        metric_tile(
            ui,
            "Node CPU",
            &format!(
                "{:.1}%",
                app.metrics_snapshot.total_node_cpu_usage_percent()
            ),
            "managed pids",
        );
        metric_tile(
            ui,
            "Node RSS",
            &format_bytes(app.metrics_snapshot.total_node_memory_bytes()),
            "resident set",
        );
    });
}
