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

        if self.fleet.nodes.is_empty() {
            if empty_state_with_action(
                ui,
                "Welcome to NeoNexus",
                "Create a local node definition to begin managing neo-cli, neo-go, or neo-rs.",
                Some("Create node"),
            ) {
                self.session.selected_view = View::Nodes;
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
        let available = ui.available_size();
        let shows_fleet =
            layout::shows_fleet_snapshot(self.session.selected_view.shows_inventory());
        let layout = layout::overview_layout(available, shows_fleet);

        if !layout::shows_selection_column(available.x, self.session.inspector_visible) {
            self.render_triage_column(ui, &layout, available.x);
            return;
        }

        ui.horizontal(|ui| {
            // The explicit gap is the whole gap: egui would otherwise add its
            // own item spacing around each allocated pane *and* around the
            // spacer, pushing the pair past the width they were sized to.
            ui.spacing_mut().item_spacing.x = 0.0;
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
            self.render_triage_column(ui, &layout, layout.right_width);
        });
    }

    /// The action queue, over the fleet snapshot when that snapshot is not
    /// already duplicated by the inventory column. Rendered on its own at
    /// narrow widths and beside the selection panel when there is room.
    fn render_triage_column(
        &mut self,
        ui: &mut egui::Ui,
        layout: &layout::OverviewLayout,
        width: f32,
    ) {
        let shows_fleet = layout.fleet_height > 0.0;
        ui.allocate_ui_with_layout(
            egui::vec2(width, layout.height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(width, layout.actions_height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        panel(ui, "Next actions", |ui| {
                            actions::render_next_actions(self, ui);
                        });
                    },
                );
                if !shows_fleet {
                    return;
                }
                ui.add_space(layout.gap);
                ui.allocate_ui_with_layout(
                    egui::vec2(width, layout.fleet_height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        panel(ui, "Fleet snapshot", |ui| {
                            fleet::render_fleet_snapshot(self, ui);
                        });
                    },
                );
            },
        );
    }
}
