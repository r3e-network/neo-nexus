use super::*;

impl NeoNexusApp {
    pub(in crate::app::views::shell::inventory) fn render_inventory_header(
        &self,
        ui: &mut egui::Ui,
    ) {
        ui.add_space(theme::SM);
        ui.vertical(|ui| {
            ui.label(theme::section_title("Inventory"));
            ui.add_space(2.0);
            ui.label(theme::muted_body("Select a local runtime"));
        });
        ui.add_space(theme::SM);
        ui.separator();
        ui.add_space(theme::SM);
    }
}
