use eframe::egui;

use crate::app::{
    domain::RemoteServerProbeRecord, text::truncate_middle, theme::muted_text, widgets::fact,
};

use super::colors::remote_probe_color;

pub(super) fn render_probe_report(ui: &mut egui::Ui, report: &RemoteServerProbeRecord) {
    ui.horizontal(|ui| {
        ui.strong("Last probe");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(report.status.label())
                    .strong()
                    .color(remote_probe_color(report.status)),
            );
        });
    });
    fact(ui, "Checked", &report.checked_at_unix.to_string());
    fact(ui, "Nodes", &node_count_summary(report));
    fact(
        ui,
        "Syncing",
        &report.syncing_nodes.unwrap_or_default().to_string(),
    );
    fact(
        ui,
        "Errors",
        &report.error_nodes.unwrap_or_default().to_string(),
    );
    fact(
        ui,
        "Blocks",
        &report.total_blocks.unwrap_or_default().to_string(),
    );
    fact(
        ui,
        "Peers",
        &report.total_peers.unwrap_or_default().to_string(),
    );
    ui.label(egui::RichText::new(truncate_middle(&report.message, 64)).color(muted_text()));
}

fn node_count_summary(report: &RemoteServerProbeRecord) -> String {
    format!(
        "{}/{}",
        report.running_nodes.unwrap_or_default(),
        report
            .total_nodes
            .or(report.public_node_count)
            .unwrap_or_default()
    )
}
