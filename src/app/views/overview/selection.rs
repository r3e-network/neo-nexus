use eframe::egui;

use super::super::super::{
    text::{short_path, truncate_middle},
    theme,
    view::View,
    widgets::{self, empty_state, fact, render_node_fact_sheet},
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
        Err(error) => {
            ui.label(
                egui::RichText::new(format!(
                    "RPC Health: {}",
                    truncate_middle(&error.to_string(), 40)
                ))
                .color(theme::danger()),
            );
        }
    }

    ui.add_space(theme::MD);
    ui.horizontal(|ui| {
        let running = node.status.is_running();
        if widgets::secondary_button_enabled(ui, "Start", !running).clicked() {
            app.start_selected_node();
        }
        if widgets::secondary_button_enabled(ui, "Stop", running).clicked() {
            app.stop_selected_node();
        }
        if widgets::secondary_button_enabled(ui, "Restart", running).clicked() {
            app.restart_selected_node();
        }
        if widgets::secondary_button(ui, "Edit").clicked() {
            app.load_selected_node_into_draft();
        }
        if widgets::secondary_button(ui, "Logs").clicked() {
            app.selected_view = View::Logs;
        }
    });
}
