mod actions;
mod facts;
mod header;
mod target;

use eframe::egui;

use crate::app::{domain::FastSyncSnapshot, NeoNexusApp};

use super::super::super::widgets::empty_state;

impl NeoNexusApp {
    pub(super) fn render_selected_snapshot_actions(
        &mut self,
        ui: &mut egui::Ui,
        snapshots: &[FastSyncSnapshot],
    ) {
        let Some(snapshot) = self
            .selected_snapshot
            .as_ref()
            .and_then(|id| snapshots.iter().find(|snapshot| &snapshot.id == id))
            .cloned()
        else {
            empty_state(
                ui,
                "No snapshot selected",
                "Choose a manifest from Registry.",
            );
            return;
        };

        header::render_snapshot_header(ui, &snapshot);
        ui.separator();
        facts::render_snapshot_facts(ui, &snapshot);
        ui.separator();
        target::render_target_node(self, ui);
        ui.add_space(8.0);
        actions::render_snapshot_actions(self, ui, &snapshot);
    }
}
