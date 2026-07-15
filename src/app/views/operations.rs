mod action_queue;
mod event_journal;
mod helpers;
mod matrix;
mod metrics;
mod readiness;
mod safety;
mod section;

use std::collections::BTreeMap;

use eframe::egui;

use crate::app::domain::{evaluate_fleet, PluginState};

use super::super::{
    theme,
    widgets::{panel, segmented_control},
    NeoNexusApp,
};

pub(in crate::app) use section::OperationsSection;

impl NeoNexusApp {
    pub(super) fn render_operations(&mut self, ui: &mut egui::Ui) {
        let plugin_states = self.plugin_states_by_node();
        let diagnostics = evaluate_fleet(&self.fleet.nodes, &plugin_states);

        metrics::render_operations_metrics(ui, &diagnostics, self.fleet.nodes.len());

        ui.add_space(theme::MD);
        let mut index = self.operations_ui.section as usize;
        let labels = OperationsSection::ALL.map(OperationsSection::label);
        if segmented_control(ui, &labels, &mut index) {
            self.operations_ui.section = OperationsSection::ALL[index];
        }
        ui.add_space(theme::MD);

        match self.operations_ui.section {
            OperationsSection::Readiness => panel(ui, "Selected readiness", |ui| {
                self.render_selected_readiness(ui, &diagnostics);
            }),
            OperationsSection::ActionQueue => panel(ui, "Action queue", |ui| {
                self.render_action_queue(ui, &diagnostics);
            }),
            OperationsSection::Ports => panel(ui, "Network port matrix", |ui| {
                self.render_port_matrix(ui, &diagnostics);
            }),
            OperationsSection::Safety => panel(ui, "Workspace safety", |ui| {
                self.render_workspace_backup(ui, &diagnostics);
            }),
            OperationsSection::Journal => panel(ui, "Event journal", |ui| {
                self.render_event_journal(ui);
            }),
        }
    }

    pub(in crate::app) fn plugin_states_by_node(&self) -> BTreeMap<String, Vec<PluginState>> {
        self.fleet.nodes
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
