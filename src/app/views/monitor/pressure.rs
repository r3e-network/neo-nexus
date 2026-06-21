use eframe::egui;

use crate::app::domain::{format_bytes, ResourcePressure};

use super::super::super::{theme::muted_text, widgets::fact, NeoNexusApp};

pub(super) fn render_system_pressure(app: &NeoNexusApp, ui: &mut egui::Ui) {
    let system = &app.metrics_snapshot.system;
    pressure_bar(ui, "CPU", system.cpu_usage_percent, system.cpu_pressure());
    pressure_bar(
        ui,
        "Memory",
        system.memory_usage_percent,
        system.memory_pressure(),
    );
    ui.separator();
    fact(
        ui,
        "Used RAM",
        &format!(
            "{} / {}",
            format_bytes(system.used_memory_bytes),
            format_bytes(system.total_memory_bytes)
        ),
    );
    fact(
        ui,
        "Available",
        &format_bytes(system.available_memory_bytes),
    );
    fact(ui, "Processes", &system.process_count.to_string());
    fact(
        ui,
        "Captured",
        &app.metrics_snapshot.captured_at_unix.to_string(),
    );
}

fn pressure_bar(ui: &mut egui::Ui, label: &str, percent: f32, pressure: ResourcePressure) {
    ui.horizontal(|ui| {
        ui.set_min_height(26.0);
        ui.label(egui::RichText::new(label).color(muted_text()));
        ui.add(
            egui::ProgressBar::new((percent / 100.0).clamp(0.0, 1.0))
                .fill(pressure_color(pressure))
                .text(format!("{percent:.0}%"))
                .desired_width(ui.available_width().max(180.0)),
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
