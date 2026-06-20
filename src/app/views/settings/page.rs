use eframe::egui;

use crate::app::{widgets::panel, NeoNexusApp};

use super::{metrics::render_settings_metrics, PANEL_GAP};

pub(super) fn render_settings(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    render_settings_metrics(app, ui);

    ui.add_space(10.0);
    let available = ui.available_size();
    let policy_width = (available.x * 0.34).clamp(330.0, 470.0);
    let upgrade_width = (available.x * 0.34).clamp(330.0, 470.0);

    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(policy_width, available.y),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                panel(ui, "Watchdog policy", |ui| {
                    app.render_watchdog_policy_settings(ui);
                });
            },
        );

        ui.add_space(PANEL_GAP);

        ui.allocate_ui_with_layout(
            egui::vec2(upgrade_width, available.y),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                panel(ui, "Runtime upgrade policy", |ui| {
                    app.render_runtime_upgrade_policy_settings(ui);
                });
            },
        );

        ui.add_space(PANEL_GAP);

        ui.allocate_ui_with_layout(
            egui::vec2(
                (available.x - policy_width - upgrade_width - PANEL_GAP * 2.0).max(300.0),
                available.y,
            ),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                render_settings_right_column(app, ui);
            },
        );
    });
}

fn render_settings_right_column(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    let available = ui.available_size();
    let storage_height = (available.y * 0.34).clamp(154.0, 178.0);
    let release_height = 122.0;

    ui.allocate_ui_with_layout(
        egui::vec2(available.x, storage_height),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            panel(ui, "Workspace storage", |ui| {
                app.render_workspace_storage_settings(ui);
            });
        },
    );

    ui.add_space(PANEL_GAP);
    ui.allocate_ui_with_layout(
        egui::vec2(available.x, release_height),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            panel(ui, "Release handoff", |ui| {
                app.render_release_package_settings(ui);
            });
        },
    );

    ui.add_space(PANEL_GAP);
    ui.allocate_ui_with_layout(
        egui::vec2(
            available.x,
            (available.y - storage_height - release_height - PANEL_GAP * 2.0).max(170.0),
        ),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            panel(ui, "Health monitors", |ui| {
                app.render_rpc_monitor_settings(ui);
            });
        },
    );
}
