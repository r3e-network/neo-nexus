use eframe::egui;

use crate::app::domain::format_bytes;

use super::super::super::{text::short_path, widgets::fact, NeoNexusApp};

impl NeoNexusApp {
    pub(super) fn render_workspace_storage_settings(&mut self, ui: &mut egui::Ui) {
        fact(ui, "Database", &short_path(self.repository.db_path(), 54));
        fact(ui, "Nodes", &short_path(&self.node_root_dir(), 54));
        fact(ui, "Reports", &short_path(&self.readiness_report_dir(), 54));
        fact(ui, "Support", &short_path(&self.support_bundle_dir(), 54));
        fact(ui, "Dist", &short_path(&self.release_package_dir(), 54));
        fact(ui, "Backups", &short_path(&self.backup_export_dir(), 54));
        fact(
            ui,
            "Runtimes",
            &short_path(&self.runtime_install_root(), 54),
        );
    }

    pub(super) fn render_release_package_settings(&mut self, ui: &mut egui::Ui) {
        let package_label = self
            .last_release_package
            .as_ref()
            .map_or_else(|| "-".to_string(), |package| package.package_id.clone());
        let package_size = self.last_release_package.as_ref().map_or_else(
            || "-".to_string(),
            |package| format_bytes(package.archive_bytes),
        );
        let verification_label = self.last_release_verification.as_ref().map_or_else(
            || "-".to_string(),
            |verification| verification.package_id.clone(),
        );

        fact(ui, "Release", &package_label);
        fact(ui, "Archive", &package_size);
        fact(ui, "Verified", &verification_label);
        ui.horizontal(|ui| {
            if ui
                .button("Package")
                .on_hover_text("Package the currently running NeoNexus executable")
                .clicked()
            {
                self.package_native_release();
            }
            if ui
                .button("Verify")
                .on_hover_text("Verify the latest package manifest, ZIP, and checksum")
                .clicked()
            {
                self.verify_native_release_package();
            }
        });
    }
}
