use crate::app::theme;
use eframe::egui;

use crate::app::{
    domain::{AlertPreviewHeader, AlertPreviewReport},
    text::truncate_middle,
    theme::muted_text,
    NeoNexusApp,
};

pub(super) fn render_alert_preview(app: &NeoNexusApp, ui: &mut egui::Ui) {
    ui.separator();
    ui.label(egui::RichText::new("Last preview").strong());
    let Some(preview) = app.last_alert_preview.as_ref() else {
        ui.label(egui::RichText::new("No preview run.").color(muted_text()));
        return;
    };

    render_preview_summary(ui, preview, app.alert_preview_matches_draft());
}

fn render_preview_summary(ui: &mut egui::Ui, preview: &AlertPreviewReport, current: bool) {
    egui::Grid::new("alert_preview_summary")
        .num_columns(2)
        .spacing([14.0, 4.0])
        .show(ui, |ui| {
            preview_status_row(ui, current);
            preview_row(ui, "Provider", &preview.provider);
            preview_row(ui, "Severity", &preview.severity);
            preview_row(ui, "Target", &preview.target);
            preview_row(ui, "Endpoint", &preview.endpoint);
            preview_row(ui, "Headers", &preview.header_count.to_string());
            preview_row(ui, "Payload", &truncate_middle(&preview.payload_json, 96));
        });

    for header in &preview.headers {
        render_header(ui, header);
    }
}

fn preview_status_row(ui: &mut egui::Ui, current: bool) {
    ui.label(egui::RichText::new("Status").color(muted_text()));
    let text = if current {
        egui::RichText::new("Current draft").color(theme::success())
    } else {
        egui::RichText::new("Draft changed").color(theme::danger())
    };
    ui.label(text);
    ui.end_row();
}

fn preview_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.label(egui::RichText::new(label).color(muted_text()));
    ui.label(value);
    ui.end_row();
}

fn render_header(ui: &mut egui::Ui, header: &AlertPreviewHeader) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Header").color(muted_text()));
        ui.label(format!("{}={}", header.name, header.value));
    });
}
