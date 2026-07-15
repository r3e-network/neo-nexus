use eframe::egui;

use crate::app::domain::{format_bytes, NodeStatus, ProcessRow};

use super::super::super::{
    format_duration,
    paging::page_count,
    text::truncate_middle,
    theme,
    view::View,
    widgets::{
        empty_state, empty_state_with_action, fact, grid_header, pagination_bar, status_badge,
    },
    NeoNexusApp, MONITOR_PROCESS_PAGE_SIZE,
};
use super::filter::render_process_filter;

pub(super) fn render_process_metrics(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    let has_process_data = !app.metrics_snapshot.node_processes.is_empty()
        || !app.metrics_snapshot.missing_processes.is_empty();
    if !has_process_data {
        if empty_state_with_action(
            ui,
            "No observed processes",
            "Start a managed node to inspect CPU, RSS, and uptime.",
            Some("Open Nodes"),
        ) {
            app.session.selected_view = View::Nodes;
        }
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

    render_selected_process_detail(app, ui, &rows);
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
            ui.label(
                theme::body(cpu_label(process.cpu_usage_percent))
                    .color(cpu_color(process.cpu_usage_percent)),
            );
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

fn render_selected_process_detail(app: &NeoNexusApp, ui: &mut egui::Ui, rows: &[ProcessRow]) {
    let Some(row) = rows
        .iter()
        .find(|row| Some(row.node_id()) == app.selected_monitor_process.as_deref())
    else {
        return;
    };

    ui.add_space(theme::SM);
    egui::Frame::new()
        .fill(theme::card_surface())
        .stroke(theme::hairline())
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(theme::label_caption("Selected process"));
            ui.add_space(theme::SM);
            match row {
                ProcessRow::Observed(process) => {
                    fact(ui, "Node", &process.node_name);
                    fact(ui, "PID", &process.pid.to_string());
                    fact(ui, "CPU", &cpu_label(process.cpu_usage_percent));
                    fact(ui, "RSS", &format_bytes(process.memory_bytes));
                    fact(
                        ui,
                        "Uptime",
                        &format_duration(std::time::Duration::from_secs(process.run_time_seconds)),
                    );
                    fact(ui, "State", &process.status);
                }
                ProcessRow::Missing(process) => {
                    fact(ui, "Node", &process.node_name);
                    fact(ui, "State", "missing / not observed");
                }
            }
        });
}

fn cpu_label(percent: f32) -> String {
    format!("{percent:.1}%")
}

fn cpu_color(percent: f32) -> egui::Color32 {
    if percent >= 90.0 {
        theme::danger()
    } else if percent >= 70.0 {
        theme::warning()
    } else {
        theme::success()
    }
}
