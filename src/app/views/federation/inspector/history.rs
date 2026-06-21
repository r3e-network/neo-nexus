use eframe::egui;

use crate::app::{
    domain::{RemoteProbeStatus, RemoteServerProbeRecord},
    paging::page_count,
    text::truncate_middle,
    theme::muted_text,
    widgets::{empty_state, pagination_bar},
    NeoNexusApp, REMOTE_PROBE_HISTORY_PAGE_SIZE,
};

use super::colors::remote_probe_color;

pub(super) fn render_probe_history(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.strong("Probe history");
    let has_history = !app.selected_remote_server_probe_history().is_empty();
    if !has_history {
        empty_state(
            ui,
            "No probe samples",
            "Probe this endpoint to record history.",
        );
        return;
    }

    render_history_filters(app, ui);
    app.clamp_remote_probe_history_page();
    let history = app.filtered_selected_remote_server_probe_history();
    if history.is_empty() {
        empty_state(
            ui,
            "No matching samples",
            "Adjust the probe history filter.",
        );
        return;
    }

    let total_pages = page_count(history.len(), REMOTE_PROBE_HISTORY_PAGE_SIZE);
    app.remote_probe_history_page = app.remote_probe_history_page.min(total_pages - 1);
    pagination_bar(
        ui,
        &mut app.remote_probe_history_page,
        total_pages,
        history.len(),
    );
    ui.separator();
    let start = app.remote_probe_history_page * REMOTE_PROBE_HISTORY_PAGE_SIZE;
    render_history_table(ui, &history, start);
}

fn render_history_filters(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Status").color(muted_text()));
        status_button(app, ui, "All", None);
        status_button(app, ui, "Healthy", Some(RemoteProbeStatus::Healthy));
        status_button(app, ui, "Degraded", Some(RemoteProbeStatus::Degraded));
        status_button(app, ui, "Disabled", Some(RemoteProbeStatus::Disabled));
        status_button(app, ui, "Unreachable", Some(RemoteProbeStatus::Unreachable));
    });
    let response = ui.add_sized(
        [ui.available_width(), 24.0],
        egui::TextEdit::singleline(&mut app.remote_probe_history_query).hint_text("Search"),
    );
    if response.changed() {
        app.remote_probe_history_page = 0;
    }
    ui.separator();
}

fn status_button(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    label: &str,
    status: Option<RemoteProbeStatus>,
) {
    if ui
        .selectable_label(app.remote_probe_history_status_filter == status, label)
        .clicked()
    {
        app.remote_probe_history_status_filter = status;
        app.remote_probe_history_page = 0;
    }
}

fn render_history_table(ui: &mut egui::Ui, history: &[RemoteServerProbeRecord], start: usize) {
    egui::Grid::new("remote_probe_history")
        .striped(true)
        .min_col_width(58.0)
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Time").color(muted_text()));
            ui.label(egui::RichText::new("Status").color(muted_text()));
            ui.label(egui::RichText::new("Nodes").color(muted_text()));
            ui.label(egui::RichText::new("Blocks").color(muted_text()));
            ui.label(egui::RichText::new("Peers").color(muted_text()));
            ui.label(egui::RichText::new("Message").color(muted_text()));
            ui.end_row();
            for record in history
                .iter()
                .skip(start)
                .take(REMOTE_PROBE_HISTORY_PAGE_SIZE)
            {
                render_history_row(ui, record);
            }
        });
}

fn render_history_row(ui: &mut egui::Ui, record: &RemoteServerProbeRecord) {
    ui.label(record.checked_at_unix.to_string());
    ui.label(egui::RichText::new(record.status.label()).color(remote_probe_color(record.status)));
    ui.label(node_summary(record))
        .on_hover_text(node_tooltip(record));
    ui.label(optional_u64(record.total_blocks));
    ui.label(optional_u64(record.total_peers));
    ui.label(truncate_middle(&non_empty(&record.message), 34));
    ui.end_row();
}

fn node_summary(record: &RemoteServerProbeRecord) -> String {
    let total = visible_total_nodes(record);
    if total == 0 && record.running_nodes.is_none() {
        return "-".to_string();
    }
    format!("{}/{}", record.running_nodes.unwrap_or_default(), total)
}

fn node_tooltip(record: &RemoteServerProbeRecord) -> String {
    format!(
        "running: {}, syncing: {}, errors: {}, total: {}",
        record.running_nodes.unwrap_or_default(),
        record.syncing_nodes.unwrap_or_default(),
        record.error_nodes.unwrap_or_default(),
        visible_total_nodes(record)
    )
}

fn visible_total_nodes(record: &RemoteServerProbeRecord) -> u64 {
    record
        .total_nodes
        .or(record.public_node_count)
        .unwrap_or_default()
}

fn optional_u64(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}

fn non_empty(value: &str) -> String {
    if value.trim().is_empty() {
        "-".to_string()
    } else {
        value.to_string()
    }
}
