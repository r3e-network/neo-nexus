use eframe::egui;

mod summary;

use crate::diagnostics::FleetDiagnostics;

use super::super::super::{text::short_path, theme::muted_text, widgets::fact, NeoNexusApp};

use summary::WorkspaceSafetySummary;

impl NeoNexusApp {
    pub(super) fn render_workspace_backup(
        &mut self,
        ui: &mut egui::Ui,
        diagnostics: &FleetDiagnostics,
    ) {
        let safety = WorkspaceSafetySummary::new(
            &self.nodes,
            &self.backup_export_dir(),
            self.workspace_integrity_report.as_ref(),
            self.last_backup_validation.as_ref(),
        );

        fact(
            ui,
            "Fleet",
            &format!("{} nodes / {}%", self.nodes.len(), diagnostics.score),
        );
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Integrity").color(muted_text()));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(&safety.integrity.status_label)
                        .strong()
                        .color(safety.integrity.color),
                );
            });
        });
        fact(ui, "Schema", &safety.integrity.schema_label);
        fact(ui, "Database", &safety.integrity.database_label);
        fact(
            ui,
            "Latest",
            &safety
                .latest_backup
                .as_ref()
                .map_or_else(|| "-".to_string(), |path| short_path(path, 36)),
        );
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Backup").color(muted_text()));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(&safety.backup_validation.status_label)
                        .strong()
                        .color(safety.backup_validation.color),
                );
            });
        });
        fact(ui, "Validated", &safety.backup_validation.counts_label);
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            if ui
                .add_enabled(safety.can_export(), egui::Button::new("Export"))
                .on_hover_text("Export workspace backup")
                .clicked()
            {
                self.export_workspace_backup();
            }
            if ui
                .add_enabled(safety.can_import(), egui::Button::new("Import"))
                .on_hover_text("Import latest workspace backup")
                .clicked()
            {
                self.import_latest_workspace_backup();
            }
            if ui
                .add_enabled(safety.can_validate_backup(), egui::Button::new("Validate"))
                .on_hover_text("Validate latest workspace backup without importing")
                .clicked()
            {
                self.validate_latest_workspace_backup();
            }
            if ui
                .button("Integrity")
                .on_hover_text("Run read-only database integrity check")
                .clicked()
            {
                self.run_workspace_integrity_check();
            }
            if ui
                .button("Bundle")
                .on_hover_text("Export redacted diagnostics support bundle")
                .clicked()
            {
                self.export_support_bundle();
            }
        });

        ui.add_space(4.0);
        ui.label(egui::RichText::new(&safety.integrity.hint).color(muted_text()));
        ui.label(egui::RichText::new(&safety.backup_validation.hint).color(muted_text()));

        if safety.node_count == 0 {
            ui.label(
                egui::RichText::new("Create a node before exporting a backup.").color(muted_text()),
            );
        } else if safety.has_running_nodes {
            ui.label(
                egui::RichText::new("Stop running nodes before importing a backup.")
                    .color(muted_text()),
            );
        }
    }
}
