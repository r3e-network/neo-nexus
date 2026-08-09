use eframe::egui;

use crate::app::domain::format_bytes;

use super::super::super::{widgets::metric_row, NeoNexusApp};

/// Resource pressure, shown inside the section it describes.
///
/// This rendered above the section tabs, so every Monitor surface opened under
/// four figures that only the Pressure tab is about — and CPU and Memory are
/// already in the status bar, visible from every view in the app.
pub(super) fn render_pressure_metrics(app: &NeoNexusApp, ui: &mut egui::Ui) {
    let system = &app.metrics_snapshot.system;
    let cpu = format!("{:.0}%", system.cpu_usage_percent);
    let memory = format!("{:.0}%", system.memory_usage_percent);
    let node_cpu = format!(
        "{:.1}%",
        app.metrics_snapshot.total_node_cpu_usage_percent()
    );
    let node_rss = format_bytes(app.metrics_snapshot.total_node_memory_bytes());

    metric_row(
        ui,
        &[
            ("CPU", &cpu, system.cpu_pressure().label()),
            ("Memory", &memory, system.memory_pressure().label()),
            ("Node CPU", &node_cpu, "managed pids"),
            ("Node RSS", &node_rss, "resident set"),
        ],
    );
}
