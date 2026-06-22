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
                ui.label(theme::section_title("Inventory"));
                ui.label(theme::muted_body("Local node resources"));
            });
        });
        ui.separator();
    }
}
