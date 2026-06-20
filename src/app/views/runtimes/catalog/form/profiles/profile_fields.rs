use super::*;

impl NeoNexusApp {
    pub(in crate::app::views::runtimes::catalog::form::profiles) fn render_runtime_catalog_profile_fields(
        &mut self,
        ui: &mut egui::Ui,
        profiles: Vec<crate::runtime::RuntimeCatalogProfile>,
        selected_profile: String,
    ) {
        labeled_text(
            ui,
            "Catalog profile",
            &mut self.runtime_catalog_profile_label,
        );
        labeled_combo(
            ui,
            "Saved",
            "runtime_catalog_profile",
            selected_profile,
            |ui| {
                if profiles.is_empty() {
                    ui.label("No saved profiles");
                }
                for profile in profiles {
                    ui.selectable_value(
                        &mut self.selected_runtime_catalog_profile,
                        Some(profile.id),
                        profile.label,
                    );
                }
            },
        );
        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                self.save_runtime_catalog_profile();
            }
            if ui.button("Recall").clicked() {
                self.load_selected_runtime_catalog_profile_into_form();
            }
            if ui.button("Delete").clicked() {
                self.delete_selected_runtime_catalog_profile();
            }
        });
    }
}
