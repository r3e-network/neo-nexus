use eframe::egui;

use crate::app::{domain::FleetDiagnostics, widgets::metric_row};

pub(super) fn render_operations_metrics(
    ui: &mut egui::Ui,
    diagnostics: &FleetDiagnostics,
    node_count: usize,
) {
    let readiness = format!("{}%", diagnostics.score);
    let ready = format!("{}/{}", diagnostics.ready_nodes, node_count);
    let critical = diagnostics.critical_count.to_string();
    let warnings = diagnostics.warning_count.to_string();

    metric_row(
        ui,
        &[
            ("Readiness", &readiness, "fleet score"),
            ("Ready", &ready, "clean nodes"),
            ("Critical", &critical, "must fix"),
            ("Warnings", &warnings, "operator review"),
        ],
    );
}
