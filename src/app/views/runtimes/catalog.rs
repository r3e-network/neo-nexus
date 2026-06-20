mod actions;
mod form;
mod summary;
mod table;

use eframe::egui;

use crate::runtime::RuntimePlatform;

use super::super::super::{widgets::empty_state, NeoNexusApp};

impl NeoNexusApp {
    pub(super) fn render_runtime_release_catalog(&mut self, ui: &mut egui::Ui) {
        self.render_runtime_catalog_profiles(ui);
        ui.separator();
        self.render_runtime_catalog_sources(ui);

        let Some(catalog) = &self.runtime_catalog else {
            empty_state(
                ui,
                "No catalog",
                "Load a local or signed HTTPS JSON runtime release catalog.",
            );
            return;
        };

        let platform = RuntimePlatform::current();
        let compatible = catalog.compatible_releases(&platform).len();
        let releases = catalog.releases.clone();
        let generated = catalog
            .generated_at_unix
            .map_or_else(|| "unknown".to_string(), |value| value.to_string());

        self.render_runtime_catalog_summary(ui, releases.len(), compatible, generated);
        self.render_runtime_catalog_table(ui, &releases, &platform);
    }
}
