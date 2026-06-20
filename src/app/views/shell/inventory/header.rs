use super::*;

impl NeoNexusApp {
    pub(in crate::app::views::shell::inventory) fn render_inventory_header(
        &self,
        ui: &mut egui::Ui,
    ) {
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.vertical(|ui| {
                ui.heading("Inventory");
                ui.label(egui::RichText::new("Local node resources").color(muted_text()));
            });
        });
        ui.separator();
    }
}
