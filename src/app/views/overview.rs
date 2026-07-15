mod actions;
mod fleet;
mod layout;
mod metrics;
mod resource;
mod selection;

use eframe::egui;

use crate::app::{domain::DashboardSummary, theme, view::View};

use super::super::{
    widgets::{callout, empty_state_with_action, panel, CalloutKind},
    NeoNexusApp,
};

impl NeoNexusApp {
    pub(super) fn render_overview(&mut self, ui: &mut egui::Ui) {
        // Shell header already shows the Home title/subtitle; keep the workspace
        // free of a second page chrome and go straight to content.

        if self.nodes.is_empty() {
            if empty_state_with_action(
                ui,
                "Welcome to NeoNexus",
                "Create a local node definition to begin managing neo-cli, neo-go, or neo-rs.",
                Some("Create node"),
            ) {
                self.selected_view = View::Nodes;
            }
            return;
        }

        let summary = DashboardSummary::load(&self.repository).ok();
        metrics::render_overview_metrics(ui, summary.as_ref());
        ui.add_space(theme::SM);
        resource::render_resource_monitor(self, ui);

        if let Some(summary) = summary.as_ref() {
            if summary.running_nodes == 0 && summary.total_nodes > 0 {
                ui.add_space(theme::SM);
                callout(
                    ui,
                    CalloutKind::Info,
                    "No nodes running",
                    "Select a node and start it from the selection panel, or resolve readiness actions below.",
                );
            }
        }

        ui.add_space(theme::MD);
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
                        egui::vec2(layout.right_width, layout.actions_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            panel(ui, "Next actions", |ui| {
                                actions::render_next_actions(self, ui);
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
