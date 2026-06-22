mod application;
mod catalog;
mod filter;
mod install;
mod inventory;
mod section;

use eframe::egui;

use crate::app::domain::{RuntimeInstallation, RuntimePlatform};

use super::super::{
    theme,
    widgets::{metric_row, panel, segmented_control},
    NeoNexusApp,
};

pub(in crate::app) use section::RuntimesSection;

impl NeoNexusApp {
    pub(super) fn render_runtimes(&mut self, ui: &mut egui::Ui) {
        let installations = self.runtime_installations();
        self.ensure_valid_runtime_selection(&installations);
        self.ensure_valid_runtime_catalog_profile_selection();
        self.ensure_valid_runtime_signer_profile_selection();
        self.ensure_valid_runtime_release_selection();

        render_runtime_metrics(ui, &installations);

        ui.add_space(theme::MD);
        let mut index = self.runtimes_section as usize;
        let labels = RuntimesSection::ALL.map(RuntimesSection::label);
        if segmented_control(ui, &labels, &mut index) {
            self.runtimes_section = RuntimesSection::ALL[index];
        }
        ui.add_space(theme::MD);

        match self.runtimes_section {
            RuntimesSection::Install => panel(ui, "Install package", |ui| {
                self.render_runtime_install_form(ui);
            }),
            RuntimesSection::Catalog => panel(ui, "Release catalog", |ui| {
                self.render_runtime_release_catalog(ui);
            }),
            RuntimesSection::Installed => panel(ui, "Installed runtimes", |ui| {
                self.render_runtime_inventory(ui, &installations);
            }),
            RuntimesSection::Applied => panel(ui, "Selected node runtime", |ui| {
                self.render_runtime_application(ui, &installations);
            }),
        }
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

    let installed = installations.len().to_string();
    let platform_label = platform.to_string();
    let compatible = platform_installs.to_string();
    let signed = signed_installs.to_string();
    metric_row(
        ui,
        &[
            ("Installed", &installed, "runtime packages"),
            ("Platform", &platform_label, "current host"),
            ("Compatible", &compatible, "for this host"),
            ("Signed", &signed, "verified packages"),
        ],
    );
}
