use eframe::egui;

use super::super::super::{
    format_duration,
    text::truncate_middle,
    theme::{self, muted_text},
    widgets::{busy_inline, columns_that_fit, fact, secondary_button, secondary_button_enabled},
    NeoNexusApp, METRICS_REFRESH_INTERVAL,
};

const MISSING_ROWS: usize = 4;

/// Narrowest a telemetry group may be before the groups stack instead.
const GROUP_MIN_WIDTH: f32 = 150.0;

/// Ten figures, in the three groups they actually belong to.
///
/// They used to be one flat column of ten `fact` rows — 220pt of a panel that
/// does not scroll, and no indication that "Pending" appeared twice meaning two
/// different things. Side by side the whole surface is a glance, and the group
/// headings say which subsystem each number is about.
pub(super) fn render_telemetry_health(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    let running_nodes = app
        .fleet
        .nodes
        .iter()
        .filter(|node| node.status.is_running())
        .count();
    let rpc = &app.async_bus.rpc_health_monitor_policy;
    let federation = &app.async_bus.remote_federation_monitor_policy;

    let groups: [(&str, Vec<(&str, String)>); 3] = [
        (
            "Processes",
            vec![
                ("Running nodes", running_nodes.to_string()),
                (
                    "Observed PIDs",
                    app.metrics_snapshot.node_processes.len().to_string(),
                ),
                (
                    "Missing PIDs",
                    app.metrics_snapshot.missing_processes.len().to_string(),
                ),
                ("Refresh", format_duration(METRICS_REFRESH_INTERVAL)),
            ],
        ),
        (
            "RPC monitor",
            vec![
                ("Automatic", enabled_label(rpc.enabled).to_string()),
                (
                    "In flight",
                    app.async_bus.rpc_health_pending.len().to_string(),
                ),
                ("Interval", format_duration(rpc.interval_duration())),
            ],
        ),
        (
            "Federation monitor",
            vec![
                ("Automatic", enabled_label(federation.enabled).to_string()),
                (
                    "In flight",
                    app.async_bus.remote_federation_pending.len().to_string(),
                ),
                ("Interval", format_duration(federation.interval_duration())),
            ],
        ),
    ];

    let per_row = columns_that_fit(ui.available_width(), GROUP_MIN_WIDTH, groups.len()).max(1);
    for chunk in groups.chunks(per_row) {
        // Always `per_row` columns, even when the last row holds fewer groups:
        // sizing that row to its own contents would stretch one group across the
        // full width and strand its values against the far edge, metres from the
        // labels they belong to.
        ui.columns(per_row, |columns| {
            for (column, (heading, rows)) in columns.iter_mut().zip(chunk) {
                column.label(theme::label_caption(*heading));
                for (label, value) in rows {
                    fact(column, label, value);
                }
            }
        });
    }

    if let Some(node_id) = app.fleet.selected_node.as_deref() {
        if app.async_bus.rpc_health_pending.contains(node_id) {
            ui.add_space(theme::SM);
            busy_inline(ui, "Checking RPC…");
        }
    }

    render_actions(app, ui);
    render_missing_processes(app, ui);
}

fn enabled_label(enabled: bool) -> &'static str {
    if enabled {
        "enabled"
    } else {
        "disabled"
    }
}

fn render_actions(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    let can_reconcile = !app.metrics_snapshot.missing_processes.is_empty();
    ui.horizontal(|ui| {
        if secondary_button(ui, "Refresh").clicked() {
            app.refresh_metrics_now();
            app.session.notice = Some("Telemetry refreshed".to_string());
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
