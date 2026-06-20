mod filter;
mod layout;
mod metrics;
mod pressure;
mod processes;
mod telemetry;

use eframe::egui;

use super::super::{widgets::panel, NeoNexusApp};

impl NeoNexusApp {
    pub(super) fn render_monitor(&mut self, ui: &mut egui::Ui) {
        metrics::render_monitor_metrics(self, ui);

        ui.add_space(10.0);
        let layout = layout::monitor_layout(ui.available_size());
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(layout.left_width, layout.height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(layout.left_width, layout.pressure_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            panel(ui, "System pressure", |ui| {
                                pressure::render_system_pressure(self, ui);
                            });
                        },
                    );
                    ui.add_space(layout.gap);
                    ui.allocate_ui_with_layout(
                        egui::vec2(layout.left_width, layout.telemetry_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            panel(ui, "Telemetry health", |ui| {
                                telemetry::render_telemetry_health(self, ui);
                            });
                        },
                    );
                },
            );

            ui.add_space(layout.gap);

            ui.allocate_ui_with_layout(
                egui::vec2(layout.process_width, layout.height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    panel(ui, "Managed processes", |ui| {
                        processes::render_process_metrics(self, ui);
                    });
                },
            );
        });
    }
}
