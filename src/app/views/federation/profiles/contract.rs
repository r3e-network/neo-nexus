use eframe::egui;

use crate::app::{
    domain::{PUBLIC_NODES_PATH, PUBLIC_STATUS_PATH, PUBLIC_SYSTEM_METRICS_PATH},
    theme::muted_text,
    widgets::fact,
};

pub(super) fn render_public_endpoint_contract(ui: &mut egui::Ui) {
    ui.separator();
    ui.label(egui::RichText::new("Public endpoint contract").color(muted_text()));
    fact(ui, "Status", PUBLIC_STATUS_PATH);
    fact(ui, "Nodes", PUBLIC_NODES_PATH);
    fact(ui, "System", PUBLIC_SYSTEM_METRICS_PATH);
}
