mod fleet;
mod layout;
mod metrics;
mod resource;
mod selection;

use eframe::egui;

use crate::dashboard::DashboardSummary;

use super::super::{widgets::panel, NeoNexusApp};

impl NeoNexusApp {
    pub(super) fn render_overview(&mut self, ui: &mut egui::Ui) {
        let summary = DashboardSummary::load(&self.repository).ok();
        metrics::render_overview_metrics(ui, summary.as_ref());

        ui.add_space(10.0);
        let layout = layout::overview_layout(ui.available_size());

        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(layout.left_width, layout.height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    panel(ui, "Current selection", |ui| {
                        selection::render_summary_selection(self, ui);
                    });
                },
            );

            ui.add_space(layout.gap);

            ui.allocate_ui_with_layout(
                egui::vec2(layout.right_width, layout.height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(layout.right_width, layout.resource_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            panel(ui, "Resource monitor", |ui| {
                                resource::render_resource_monitor(self, ui);
                            });
                        },
                    );
                    ui.add_space(layout.gap);
                    ui.allocate_ui_with_layout(
                        egui::vec2(layout.right_width, layout.fleet_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            panel(ui, "Fleet snapshot", |ui| {
                                fleet::render_fleet_snapshot(self, ui);
                            });
                        },
                    );
                },
            );
        });
    }
}
