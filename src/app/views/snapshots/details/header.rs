use eframe::egui;

use crate::app::{domain::FastSyncSnapshot, text::truncate_middle, theme::section_title};

use super::super::status::{snapshot_status_color, status_label};

pub(super) fn render_snapshot_header(ui: &mut egui::Ui, snapshot: &FastSyncSnapshot) {
    ui.horizontal(|ui| {
        ui.label(section_title(truncate_middle(&snapshot.label, 34)).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(status_label(snapshot))
                    .strong()
                    .color(snapshot_status_color(snapshot)),
            );
        });
    });
}
