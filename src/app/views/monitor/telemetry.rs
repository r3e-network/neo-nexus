use eframe::egui;

use super::super::super::{
    format_duration,
    text::truncate_middle,
    theme::muted_text,
    widgets::{fact, secondary_button, secondary_button_enabled},
    NeoNexusApp, METRICS_REFRESH_INTERVAL,
};

const MISSING_ROWS: usize = 4;

pub(super) fn render_telemetry_health(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    let running_nodes = app
        .nodes
        .iter()
        .filter(|node| node.status.is_running())
        .count();
    fact(ui, "Running nodes", &running_nodes.to_string());
    fact(
        ui,
        "Observed PIDs",
        &app.metrics_snapshot.node_processes.len().to_string(),
    );
    fact(
        ui,
        "Missing PIDs",
        &app.metrics_snapshot.missing_processes.len().to_string(),
    );
    fact(ui, "Refresh", &format_duration(METRICS_REFRESH_INTERVAL));
    fact(
        ui,
        "RPC Auto",
        if app.rpc_health_monitor_policy.enabled {
            "enabled"
        } else {
            "disabled"
        },
    );
    fact(ui, "RPC Pending", &app.rpc_health_pending.len().to_string());
    fact(
        ui,
        "RPC Interval",
        &format_duration(app.rpc_health_monitor_policy.interval_duration()),
    );
    fact(
        ui,
        "Fed Auto",
        if app.remote_federation_monitor_policy.enabled {
            "enabled"
        } else {
            "disabled"
        },
    );
    fact(
        ui,
        "Fed Pending",
        &app.remote_federation_pending.len().to_string(),
    );
    fact(
        ui,
        "Fed Interval",
        &format_duration(app.remote_federation_monitor_policy.interval_duration()),
    );

    render_actions(app, ui);
    render_missing_processes(app, ui);
}

fn render_actions(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    let can_reconcile = !app.metrics_snapshot.missing_processes.is_empty();
    ui.horizontal(|ui| {
        if secondary_button(ui, "Refresh").clicked() {
            app.refresh_metrics_now();
            app.notice = Some("Telemetry refreshed".to_string());
        }
        if secondary_button_enabled(ui, "Focus Missing", can_reconcile)
            .on_hover_text("Show missing running-node PIDs in the process table")
            .clicked()
        {
            app.focus_missing_processes();
        }
        if secondary_button_enabled(ui, "Clear Filters", app.has_active_monitor_process_filter())
            .on_hover_text("Show all managed process rows")
            .clicked()
        {
            app.clear_monitor_process_filters();
        }
        if secondary_button_enabled(ui, "Repair Missing", can_reconcile)
            .on_hover_text("Mark missing running process records as stopped")
            .clicked()
        {
            app.reconcile_missing_process_records();
        }
    });
}

fn render_missing_processes(app: &NeoNexusApp, ui: &mut egui::Ui) {
    ui.separator();
    if app.metrics_snapshot.missing_processes.is_empty() {
        ui.label(egui::RichText::new("No missing running processes.").color(muted_text()));
        return;
    }

    ui.label(
        egui::RichText::new("Missing running-node PIDs need review before repair.")
            .color(muted_text()),
    );
    for row in 0..MISSING_ROWS {
        if let Some(missing) = app.metrics_snapshot.missing_processes.get(row) {
            ui.horizontal(|ui| {
                ui.label(truncate_middle(&missing.node_name, 20));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("PID {}", missing.pid));
                });
            });
        } else {
            ui.label(" ");
        }
    }
}
