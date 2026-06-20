use eframe::egui;

use crate::{
    metrics::{format_bytes, ProcessRow},
    types::NodeStatus,
};

use super::super::super::{
    format_duration,
    paging::page_count,
    text::truncate_middle,
    theme::{muted_text, status_color},
    widgets::{empty_state, pagination_bar},
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
    ui.separator();

    let start = app.monitor_process_page * MONITOR_PROCESS_PAGE_SIZE;
    egui::Grid::new("monitor_process_metrics")
        .striped(true)
        .min_col_width(66.0)
        .show(ui, |ui| {
            ui.strong("Node");
            ui.strong("PID");
            ui.strong("CPU");
            ui.strong("RSS");
            ui.strong("Uptime");
            ui.strong("State");
            ui.end_row();

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
                app.selected_node = Some(process.node_id.clone());
            }
            ui.label(process.pid.to_string());
            ui.label(cpu_label(process.cpu_usage_percent));
            ui.label(format_bytes(process.memory_bytes));
            ui.label(format_duration(std::time::Duration::from_secs(
                process.run_time_seconds,
            )));
            ui.label(
                egui::RichText::new(truncate_middle(&process.status, 14))
                    .color(status_color(NodeStatus::Running))
                    .strong(),
            );
        }
        ProcessRow::Missing(process) => {
            if ui
                .selectable_label(selected, truncate_middle(&process.node_name, 24))
                .clicked()
            {
                app.selected_monitor_process = Some(process.node_id.clone());
                app.selected_node = Some(process.node_id.clone());
            }
            ui.label(process.pid.to_string());
            ui.label(egui::RichText::new("-").color(muted_text()));
            ui.label(egui::RichText::new("-").color(muted_text()));
            ui.label(egui::RichText::new("-").color(muted_text()));
            ui.label(
                egui::RichText::new("missing")
                    .color(status_color(NodeStatus::Error))
                    .strong(),
            );
        }
    }
}

fn cpu_label(value: f32) -> egui::RichText {
    let label = format!("{value:.1}%");
    if value >= 80.0 {
        egui::RichText::new(label)
            .color(status_color(NodeStatus::Error))
            .strong()
    } else if value >= 50.0 {
        egui::RichText::new(label)
            .color(status_color(NodeStatus::Starting))
            .strong()
    } else {
        egui::RichText::new(label)
    }
}
