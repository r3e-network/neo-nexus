use std::path::Path;

use eframe::egui;

use crate::app::domain::{LogReader, LogSnapshot, NodeConfig, NodeStatus};

use super::{
    super::super::{
        text::{short_path, truncate_middle},
        theme::{section_title, status_color},
        widgets::{fact, render_node_fact_sheet, secondary_button, secondary_button_enabled},
        NeoNexusApp,
    },
    diagnosis::{diagnosis_color, retention_label},
};

pub(super) fn render_log_context(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    node: &NodeConfig,
    path: &Path,
    snapshot: &anyhow::Result<LogSnapshot>,
) {
    render_log_heading(ui, node);
    ui.separator();
    render_node_fact_sheet(ui, node);
    ui.separator();
    fact(ui, "File", &short_path(path, 42));
    render_snapshot_facts(app, ui, snapshot);
    ui.add_space(8.0);
    render_log_controls(app, ui);
}

fn render_log_heading(ui: &mut egui::Ui, node: &NodeConfig) {
    ui.horizontal(|ui| {
        ui.label(section_title(truncate_middle(&node.name, 24)).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(node.status.label())
                    .color(status_color(node.status))
                    .strong(),
            );
        });
    });
}

fn render_snapshot_facts(
    app: &NeoNexusApp,
    ui: &mut egui::Ui,
    snapshot: &anyhow::Result<LogSnapshot>,
) {
    match snapshot {
        Ok(snapshot) => {
            let diagnosis = LogReader::diagnose(snapshot);
            fact(ui, "Bytes", &snapshot.bytes.to_string());
            fact(ui, "Lines", &snapshot.lines.len().to_string());
            if !app.log_query.trim().is_empty() {
                let matches = LogReader::filtered_lines(snapshot, &app.log_query).len();
                fact(ui, "Matches", &matches.to_string());
            }
            fact(ui, "Window", retention_label(snapshot));
            fact(ui, "Diagnosis", diagnosis.status.label());
            ui.label(
                egui::RichText::new(truncate_middle(&diagnosis.summary, 56))
                    .color(diagnosis_color(diagnosis.status)),
            );
            if let Some(action) = diagnosis.recommendations.first() {
                fact(ui, "Action", &truncate_middle(action, 52));
            }
        }
        Err(error) => {
            fact(ui, "State", "Unreadable");
            ui.label(
                egui::RichText::new(truncate_middle(&error.to_string(), 56))
                    .color(status_color(NodeStatus::Error)),
            );
        }
    }
}

fn render_log_controls(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("Search");
        let response = ui.add_sized(
            [ui.available_width().max(140.0), 24.0],
            egui::TextEdit::singleline(&mut app.log_query),
        );
        if response.changed() {
            app.log_page = 0;
            app.log_follow_tail = false;
        }
    });
    ui.horizontal(|ui| {
        if secondary_button(ui, "Refresh").clicked() {
            app.notice = Some("Log refreshed".to_string());
        }
        if secondary_button(ui, "Latest").clicked() {
            app.log_follow_tail = true;
        }
        ui.checkbox(&mut app.log_follow_tail, "Follow Tail");
    });
    ui.horizontal(|ui| {
        if secondary_button_enabled(ui, "Clear Search", !app.log_query.trim().is_empty()).clicked()
        {
            app.log_query.clear();
            app.log_page = 0;
        }
        if secondary_button(ui, "Clear Log").clicked() {
            app.clear_selected_log();
        }
    });
}
