use eframe::egui;

use crate::app::{domain::FleetDiagnostics, widgets::metric_tile};

pub(super) fn render_operations_metrics(
    ui: &mut egui::Ui,
    diagnostics: &FleetDiagnostics,
    node_count: usize,
) {
    ui.horizontal(|ui| {
        metric_tile(
            ui,
            "Readiness",
            &format!("{}%", diagnostics.score),
            "fleet score",
        );
        metric_tile(
            ui,
            "Ready",
            &format!("{}/{}", diagnostics.ready_nodes, node_count),
            "clean nodes",
        );
        metric_tile(
            ui,
            "Critical",
            &diagnostics.critical_count.to_string(),
            "must fix",
        );
        metric_tile(
            ui,
            "Warnings",
            &diagnostics.warning_count.to_string(),
            "operator review",
        );
    });
}
