use super::*;

impl NeoNexusApp {
    pub(in crate::app::views::shell::inventory) fn render_inventory_header(
        &self,
        ui: &mut egui::Ui,
    ) {
        ui.add_space(theme::MD);
        ui.horizontal(|ui| {
            ui.add_space(theme::MD);
            ui.vertical(|ui| {
                ui.label(theme::section_title("Inventory"));
                ui.label(theme::muted_body("Local node resources"));
            });
        });
        ui.separator();
    }
}
