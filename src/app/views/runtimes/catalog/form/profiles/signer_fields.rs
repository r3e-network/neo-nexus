use super::*;

impl NeoNexusApp {
    pub(in crate::app::views::runtimes::catalog::form::profiles) fn render_runtime_signer_profile_fields(
        &mut self,
        ui: &mut egui::Ui,
        signer_profiles: Vec<crate::runtime::RuntimeSignerProfile>,
        selected_signer: String,
    ) {
        labeled_text(ui, "Signer", &mut self.runtime_signer_profile_label);
        labeled_combo(
            ui,
            "Trusted",
            "runtime_signer_profile",
            selected_signer,
            |ui| {
                if signer_profiles.is_empty() {
                    ui.label("No trusted signers");
                }
                for profile in signer_profiles {
                    let label = if profile.enabled {
                        profile.label
                    } else {
                        format!("{} (disabled)", profile.label)
                    };
                    ui.selectable_value(
                        &mut self.selected_runtime_signer_profile,
                        Some(profile.id),
                        label,
                    );
                }
            },
        );
        labeled_text(ui, "Signer key", &mut self.runtime_signer_public_key);
        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                self.save_runtime_signer_profile();
            }
            if ui.button("Catalog").clicked() {
                self.use_selected_runtime_signer_for_catalog();
            }
            if ui.button("Package").clicked() {
                self.use_selected_runtime_signer_for_package();
            }
            if ui.button("Delete").clicked() {
                self.delete_selected_runtime_signer_profile();
            }
        });
    }
}
