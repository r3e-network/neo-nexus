use eframe::egui;

use super::super::super::{
    text::{short_path, truncate_middle},
    theme,
    widgets::{empty_state, fact, render_node_fact_sheet, status_badge, toolbar, ToolbarAction},
    NeoNexusApp,
};
// High-level core read so the view does not reach into the repository during paint.
use crate::app::domain::latest_node_rpc_health;

pub(super) fn render_summary_selection(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    let Some(node) = app.selected_node().cloned() else {
        empty_state(ui, "No node selected", "Choose a node from Inventory.");
        return;
    };

    ui.horizontal(|ui| {
        ui.label(theme::section_title(truncate_middle(&node.name, 28)));
        ui.add_space(theme::SM);
        status_badge(ui, node.status);
    });
    ui.add_space(theme::SM);
    ui.separator();
    ui.add_space(theme::SM);

    render_node_fact_sheet(ui, &node);
    fact(ui, "Binary", &short_path(&node.binary_path, 42));
    if let Some(pid) = node.pid {
        fact(ui, "PID", &pid.to_string());
    }
    match latest_node_rpc_health(&app.repository, &node.id) {
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
    let running = node.status.is_running();
    let actions = [
        ToolbarAction::primary("start", "Start")
            .enabled(!running)
            .hint("Start the selected stopped node"),
        ToolbarAction::secondary("stop", "Stop")
            .enabled(running)
            .hint("Stop the selected running node"),
        ToolbarAction::secondary("restart", "Restart")
            .enabled(running)
            .hint("Restart the selected running node"),
        ToolbarAction::secondary("edit", "Edit").hint("Open Node Studio with this definition"),
        ToolbarAction::secondary("logs", "Logs").hint("Open runtime logs for this node"),
    ];
    if let Some(id) = toolbar(ui, &actions) {
        match id {
            "start" => app.start_selected_node(),
            "stop" => app.stop_selected_node(),
            "restart" => app.restart_selected_node(),
            "edit" => {
                app.load_selected_node_into_draft();
                app.open_node_workspace_tab(crate::app::views::NodeWorkspaceTab::Studio);
            }
            "logs" => app.open_node_workspace_tab(crate::app::views::NodeWorkspaceTab::Logs),
            _ => {}
        }
    }
}
