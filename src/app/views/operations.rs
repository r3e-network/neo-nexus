mod action_queue;
mod event_journal;
mod helpers;
mod layout;
mod matrix;
mod metrics;
mod readiness;
mod safety;

use std::collections::BTreeMap;

use eframe::egui;

use crate::app::domain::{evaluate_fleet, PluginState};

use super::super::{widgets::panel, NeoNexusApp};

impl NeoNexusApp {
    pub(super) fn render_operations(&mut self, ui: &mut egui::Ui) {
        let plugin_states = self.plugin_states_by_node();
        let diagnostics = evaluate_fleet(&self.nodes, &plugin_states);

        metrics::render_operations_metrics(ui, &diagnostics, self.nodes.len());

        ui.add_space(10.0);
        let operations_layout = layout::operations_layout(ui.available_size());

        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(
                    operations_layout.readiness_width,
                    operations_layout.top_height,
                ),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    panel(ui, "Selected readiness", |ui| {
                        self.render_selected_readiness(ui, &diagnostics);
                    });
                },
            );

            ui.add_space(layout::PANEL_GAP);

            ui.allocate_ui_with_layout(
                egui::vec2(operations_layout.action_width, operations_layout.top_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    panel(ui, "Action queue", |ui| {
                        self.render_action_queue(ui, &diagnostics);
                    });
                },
            );
        });

        ui.add_space(layout::PANEL_GAP);
        let bottom_layout = layout::operations_bottom_layout(ui.available_size());

        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(bottom_layout.matrix_width, bottom_layout.height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    panel(ui, "Network port matrix", |ui| {
                        self.render_port_matrix(ui, &diagnostics);
                    });
                },
            );

            ui.add_space(layout::PANEL_GAP);

            ui.allocate_ui_with_layout(
                egui::vec2(bottom_layout.side_width, bottom_layout.height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    let side_layout = layout::operations_side_layout(ui.available_size());
                    ui.allocate_ui_with_layout(
                        egui::vec2(side_layout.width, side_layout.safety_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            panel(ui, "Workspace safety", |ui| {
                                self.render_workspace_backup(ui, &diagnostics);
                            });
                        },
                    );
                    ui.add_space(layout::PANEL_GAP);
                    ui.allocate_ui_with_layout(
                        egui::vec2(side_layout.width, side_layout.journal_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            panel(ui, "Event journal", |ui| {
                                self.render_event_journal(ui);
                            });
                        },
                    );
                },
            );
        });
    }

    fn plugin_states_by_node(&self) -> BTreeMap<String, Vec<PluginState>> {
        self.nodes
            .iter()
            .map(|node| {
                (
                    node.id.clone(),
                    self.repository
                        .list_plugin_states(&node.id)
                        .unwrap_or_default(),
                )
            })
            .collect()
    }
}
