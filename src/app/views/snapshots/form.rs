use eframe::egui;

use crate::types::{Network, NodeType};

use super::super::super::{
    theme::muted_text,
    widgets::{labeled_combo, labeled_text},
    NeoNexusApp,
};

impl NeoNexusApp {
    pub(super) fn render_snapshot_manifest_form(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Local or HTTPS snapshot source").color(muted_text()));
        ui.separator();

        labeled_text(ui, "ID", &mut self.snapshot_draft.id);
        labeled_text(ui, "Label", &mut self.snapshot_draft.label);
        labeled_combo(
            ui,
            "Network",
            "snapshot_network",
            self.snapshot_draft.network.to_string(),
            |ui| {
                for network in Network::ALL {
                    ui.selectable_value(
                        &mut self.snapshot_draft.network,
                        network,
                        network.to_string(),
                    );
                }
            },
        );
        labeled_combo(
            ui,
            "Runtime",
            "snapshot_node_type",
            self.snapshot_draft.node_type.to_string(),
            |ui| {
                for node_type in NodeType::ALL {
                    ui.selectable_value(
                        &mut self.snapshot_draft.node_type,
                        node_type,
                        node_type.to_string(),
                    );
                }
            },
        );
        labeled_text(ui, "Source", &mut self.snapshot_draft.source_path);
        labeled_text(ui, "HTTPS URL", &mut self.snapshot_draft.source_url);
        labeled_text(ui, "File name", &mut self.snapshot_draft.download_file_name);
        ui.horizontal(|ui| {
            ui.label("Max");
            ui.add(
                egui::DragValue::new(&mut self.snapshot_draft.download_max_mib)
                    .range(1..=1_048_576)
                    .suffix(" MiB")
                    .speed(128.0),
            );
        });
        labeled_text(ui, "SHA-256", &mut self.snapshot_draft.expected_sha256);

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button("Save Manifest").clicked() {
                self.save_snapshot_manifest();
            }
            if ui.button("Download HTTPS").clicked() {
                self.download_snapshot_from_draft();
            }
            if ui.button("Reset").clicked() {
                self.snapshot_draft = Default::default();
                self.notice = Some("Snapshot draft reset".to_string());
            }
        });
    }
}
