mod application;
mod catalog;
mod filter;
mod install;
mod inventory;

use eframe::egui;

use crate::runtime::{RuntimeInstallation, RuntimePlatform};

use super::super::{
    widgets::{metric_tile, panel},
    NeoNexusApp,
};

const PANEL_GAP: f32 = 8.0;

impl NeoNexusApp {
    pub(super) fn render_runtimes(&mut self, ui: &mut egui::Ui) {
        let installations = self.runtime_installations();
        self.ensure_valid_runtime_selection(&installations);
        self.ensure_valid_runtime_catalog_profile_selection();
        self.ensure_valid_runtime_signer_profile_selection();
        self.ensure_valid_runtime_release_selection();

        render_runtime_metrics(ui, &installations);

        ui.add_space(10.0);
        let available = ui.available_size();
        let left_width = (available.x * 0.38).clamp(340.0, 470.0);
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(left_width, available.y),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    panel(ui, "Install package", |ui| {
                        self.render_runtime_install_form(ui);
                    });
                },
            );

            ui.add_space(PANEL_GAP);

            ui.allocate_ui_with_layout(
                egui::vec2(
                    (available.x - left_width - PANEL_GAP).max(420.0),
                    available.y,
                ),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    let catalog_height = (ui.available_height() * 0.58).clamp(420.0, 540.0);
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), catalog_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            panel(ui, "Release catalog", |ui| {
                                self.render_runtime_release_catalog(ui);
                            });
                        },
                    );
                    ui.add_space(PANEL_GAP);
                    let inventory_height = (ui.available_height() * 0.52).max(170.0);
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), inventory_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            panel(ui, "Installed runtimes", |ui| {
                                self.render_runtime_inventory(ui, &installations);
                            });
                        },
                    );
                    ui.add_space(PANEL_GAP);
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), ui.available_height()),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            panel(ui, "Selected node runtime", |ui| {
                                self.render_runtime_application(ui, &installations);
                            });
                        },
                    );
                },
            );
        });
    }
}

fn render_runtime_metrics(ui: &mut egui::Ui, installations: &[RuntimeInstallation]) {
    let platform = RuntimePlatform::current();
    let platform_installs = installations
        .iter()
        .filter(|installation| installation.platform == platform)
        .count();
    let signed_installs = installations
        .iter()
        .filter(|installation| installation.signature_verified)
        .count();

    ui.horizontal(|ui| {
        metric_tile(
            ui,
            "Installed",
            &installations.len().to_string(),
            "runtime packages",
        );
        metric_tile(ui, "Platform", &platform.to_string(), "current host");
        metric_tile(
            ui,
            "Compatible",
            &platform_installs.to_string(),
            "for this host",
        );
        metric_tile(
            ui,
            "Signed",
            &signed_installs.to_string(),
            "verified packages",
        );
    });
}
