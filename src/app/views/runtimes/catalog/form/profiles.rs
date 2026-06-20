use super::*;

mod profile_fields;
mod signer_fields;

impl NeoNexusApp {
    pub(in crate::app::views::runtimes::catalog) fn render_runtime_catalog_profiles(
        &mut self,
        ui: &mut egui::Ui,
    ) {
        let profiles = self.runtime_catalog_profiles.clone();
        let selected_profile = self
            .selected_runtime_catalog_profile()
            .map_or_else(|| "No saved profile".to_string(), |profile| profile.label);
        let signer_profiles = self.runtime_signer_profiles.clone();
        let selected_signer = self.selected_runtime_signer_profile().map_or_else(
            || "No trusted signer".to_string(),
            |profile| {
                if profile.enabled {
                    profile.label
                } else {
                    format!("{} (disabled)", profile.label)
                }
            },
        );

        ui.columns(2, |columns| {
            self.render_runtime_catalog_profile_fields(&mut columns[0], profiles, selected_profile);
            self.render_runtime_signer_profile_fields(
                &mut columns[1],
                signer_profiles,
                selected_signer,
            );
        });
    }
}
