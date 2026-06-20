use eframe::egui;

use crate::metrics::{format_bytes, ResourcePressure};

use super::super::super::{theme::muted_text, widgets::fact, NeoNexusApp};

pub(super) fn render_resource_monitor(app: &NeoNexusApp, ui: &mut egui::Ui) {
    let system = &app.metrics_snapshot.system;
    resource_bar(ui, "CPU", system.cpu_usage_percent, system.cpu_pressure());
    resource_bar(
        ui,
        "Memory",
        system.memory_usage_percent,
        system.memory_pressure(),
    );
    fact(
        ui,
        "RAM",
        &format!(
            "{} / {}",
            format_bytes(system.used_memory_bytes),
            format_bytes(system.total_memory_bytes)
        ),
    );
    fact(
        ui,
        "Node CPU",
        &format!(
            "{:.1}%",
            app.metrics_snapshot.total_node_cpu_usage_percent()
        ),
    );
    fact(
        ui,
        "Node RSS",
        &format_bytes(app.metrics_snapshot.total_node_memory_bytes()),
    );
    fact(ui, "Processes", &system.process_count.to_string());
}

fn resource_bar(ui: &mut egui::Ui, label: &str, percent: f32, pressure: ResourcePressure) {
    ui.horizontal(|ui| {
        ui.set_min_height(24.0);
        ui.label(egui::RichText::new(label).color(muted_text()));
        ui.add(
            egui::ProgressBar::new((percent / 100.0).clamp(0.0, 1.0))
                .fill(pressure_color(pressure))
                .text(format!("{percent:.0}%"))
                .desired_width(ui.available_width().max(160.0)),
        );
    });
}

fn pressure_color(pressure: ResourcePressure) -> egui::Color32 {
    match pressure {
        ResourcePressure::Nominal => egui::Color32::from_rgb(21, 128, 61),
        ResourcePressure::Elevated => egui::Color32::from_rgb(202, 138, 4),
        ResourcePressure::Critical => egui::Color32::from_rgb(185, 28, 28),
    }
}
