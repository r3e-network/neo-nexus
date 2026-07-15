use eframe::egui;

use crate::app::domain::{format_bytes, NodeStatus, ProcessRow};

use super::super::super::{
    format_duration,
    paging::page_count,
    text::truncate_middle,
    theme,
    widgets::{empty_state, grid_header, pagination_bar, status_badge},
    NeoNexusApp, MONITOR_PROCESS_PAGE_SIZE,
};
use super::filter::render_process_filter;

pub(super) fn render_process_metrics(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    let has_process_data = !app.metrics_snapshot.node_processes.is_empty()
        || !app.metrics_snapshot.missing_processes.is_empty();
    if !has_process_data {
        empty_state(
            ui,
            "No observed processes",
            "Start a managed node to inspect metrics.",
        );
        return;
    }

    render_process_filter(app, ui);
    app.ensure_valid_monitor_process_selection();
    let rows = app.filtered_monitor_process_rows();
    if rows.is_empty() {
        empty_state(ui, "No matching processes", "Adjust the process filter.");
        return;
    }

    let total_pages = page_count(rows.len(), MONITOR_PROCESS_PAGE_SIZE);
    app.monitor_process_page = app.monitor_process_page.min(total_pages - 1);
    pagination_bar(ui, &mut app.monitor_process_page, total_pages, rows.len());
    ui.add_space(theme::SM);

    let start = app.monitor_process_page * MONITOR_PROCESS_PAGE_SIZE;
    egui::Grid::new("monitor_process_metrics")
        .striped(true)
        .min_col_width(66.0)
        .show(ui, |ui| {
            grid_header(ui, &["Node", "PID", "CPU", "RSS", "Uptime", "State"]);

            for row in rows.iter().skip(start).take(MONITOR_PROCESS_PAGE_SIZE) {
                render_process_row(app, ui, row);
                ui.end_row();
            }
        });
}

fn render_process_row(app: &mut NeoNexusApp, ui: &mut egui::Ui, row: &ProcessRow) {
    let selected = app.selected_monitor_process.as_deref() == Some(row.node_id());
    match row {
        ProcessRow::Observed(process) => {
            if ui
                .selectable_label(selected, truncate_middle(&process.node_name, 24))
                .clicked()
            {
                app.selected_monitor_process = Some(process.node_id.clone());
                app.fleet.selected_node = Some(process.node_id.clone());
            }
            ui.label(process.pid.to_string());
            ui.label(cpu_label(process.cpu_usage_percent));
            ui.label(format_bytes(process.memory_bytes));
            ui.label(format_duration(std::time::Duration::from_secs(
                process.run_time_seconds,
            )));
            status_badge(ui, NodeStatus::Running);
        }
        ProcessRow::Missing(process) => {
            if ui
                .selectable_label(selected, truncate_middle(&process.node_name, 24))
                .clicked()
            {
                app.selected_monitor_process = Some(process.node_id.clone());
                app.fleet.selected_node = Some(process.node_id.clone());
            }
            ui.label("—");
            ui.label("—");
            ui.label("—");
            ui.label("—");
            status_badge(ui, NodeStatus::Error);
        }
    }
}

fn cpu_label(percent: f32) -> String {
    format!("{percent:.1}%")
}
