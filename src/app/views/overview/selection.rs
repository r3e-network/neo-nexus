use eframe::egui;

use crate::types::NodeStatus;

use super::super::super::{
    text::{short_path, truncate_middle},
    view::View,
    widgets::{empty_state, fact, render_node_fact_sheet},
    NeoNexusApp,
};

pub(super) fn render_summary_selection(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    let Some(node) = app.selected_node().cloned() else {
        empty_state(ui, "No node selected", "Choose a node from Inventory.");
        return;
    };

    render_node_fact_sheet(ui, &node);
    fact(ui, "Binary", &short_path(&node.binary_path, 42));
    fact(
        ui,
        "Status",
        &format!(
            "{}{}",
            node.status,
            node.pid.map_or(String::new(), |pid| format!(" ({pid})"))
        ),
    );
    match app.repository.latest_rpc_health(&node.id) {
        Ok(Some(health)) => {
            fact(ui, "RPC Health", health.status.label());
            fact(
                ui,
                "RPC Height",
                &health
                    .block_count
                    .map_or_else(|| "-".to_string(), |height| height.to_string()),
            );
        }
        Ok(None) => fact(ui, "RPC Health", "unchecked"),
        Err(error) => fact(ui, "RPC Health", &truncate_middle(&error.to_string(), 40)),
    }

    ui.add_space(10.0);
    ui.horizontal(|ui| {
        let running = node.status == NodeStatus::Running;
        if ui
            .add_enabled(!running, egui::Button::new("Start"))
            .clicked()
        {
            app.start_selected_node();
        }
        if ui.add_enabled(running, egui::Button::new("Stop")).clicked() {
            app.stop_selected_node();
        }
        if ui
            .add_enabled(running, egui::Button::new("Restart"))
            .clicked()
        {
            app.restart_selected_node();
        }
        if ui.button("Edit").clicked() {
            app.load_selected_node_into_draft();
        }
        if ui.button("Logs").clicked() {
            app.selected_view = View::Logs;
        }
    });
}
