use eframe::egui;

use crate::app::domain::format_bytes;

use super::super::super::{
    text::short_path,
    theme::{self, UiDensity},
    widgets::{fact, filter_chip, primary_button, secondary_button},
    NeoNexusApp,
};

impl NeoNexusApp {
    pub(super) fn render_workspace_storage_settings(&mut self, ui: &mut egui::Ui) {
        ui.label(theme::label_caption("Workspace paths"));
        ui.add_space(theme::SM);
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

        ui.add_space(theme::MD);
        ui.label(theme::label_caption("Appearance"));
        ui.add_space(theme::SM);
        fact(
            ui,
            "Theme",
            if self.session.theme.is_dark() {
                "Dark (toggle in sidebar)"
            } else {
                "Light (toggle in sidebar)"
            },
        );
        ui.horizontal(|ui| {
            ui.label(theme::muted_body("Density"));
            ui.add_space(theme::SM);
            for density in [UiDensity::Comfortable, UiDensity::Compact] {
                if filter_chip(
                    ui,
                    density.label(),
                    self.session.density == density,
                ) {
                    self.set_ui_density(density);
                }
            }
        });
        ui.label(theme::muted_body(
            "Compact tightens buttons/spacing and uses single-line inventory rows; chrome stays fixed.",
        ));
    }

    pub(super) fn render_release_package_settings(&mut self, ui: &mut egui::Ui) {
        ui.label(theme::label_caption("Latest package"));
        ui.add_space(theme::SM);
        let package_label = self
            .last_release_package
            .as_ref()
            .map_or_else(|| "—".to_string(), |package| package.package_id.clone());
        let package_size = self.last_release_package.as_ref().map_or_else(
            || "—".to_string(),
            |package| format_bytes(package.archive_bytes),
        );
        let verification_label = self.last_release_verification.as_ref().map_or_else(
            || "—".to_string(),
            |verification| verification.package_id.clone(),
        );

        fact(ui, "Release", &package_label);
        fact(ui, "Archive", &package_size);
        fact(ui, "Verified", &verification_label);
        ui.add_space(theme::MD);
        ui.horizontal(|ui| {
            if primary_button(ui, "Package")
                .on_hover_text("Package the currently running NeoNexus executable")
                .clicked()
            {
                self.package_native_release();
            }
            if secondary_button(ui, "Verify")
                .on_hover_text("Verify the latest package manifest, ZIP, and checksum")
                .clicked()
            {
                self.verify_native_release_package();
            }
        });
    }
}
