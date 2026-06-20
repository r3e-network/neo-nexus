use super::*;

impl NeoNexusApp {
    pub(in crate::app::views::runtimes::catalog) fn render_runtime_catalog_sources(
        &mut self,
        ui: &mut egui::Ui,
    ) {
        labeled_text(ui, "Catalog", &mut self.runtime_catalog_source);
        labeled_text(ui, "Signature", &mut self.runtime_catalog_signature_source);
        labeled_text(ui, "Catalog key", &mut self.runtime_catalog_public_key);
        ui.horizontal(|ui| {
            if ui.button("Load").clicked() {
                self.load_runtime_release_catalog();
            }
            if ui.button("Use Release").clicked() {
                self.load_selected_runtime_release_into_draft();
            }
            if ui.button("Use Latest").clicked() {
                self.use_latest_runtime_release_for_selected_node();
            }
        });
    }
}
