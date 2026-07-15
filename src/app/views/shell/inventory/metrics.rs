use super::*;

impl NeoNexusApp {
    pub(in crate::app::views::shell::inventory) fn render_inventory_metrics(
        &self,
        ui: &mut egui::Ui,
    ) {
        let running = self.fleet
            .nodes
            .iter()
            .filter(|node| node.status.is_running())
            .count();
        let stopped = self.fleet
            .nodes
            .iter()
            .filter(|node| node.status.is_stopped())
            .count();
        let filter = self.node_inventory_filter();
        let visible = if filter.is_empty() {
            self.fleet.nodes.len()
        } else {
            self.filtered_inventory_nodes().len()
        };

        ui.horizontal(|ui| {
            ui.add_space(theme::MD);
            // Two rows of two compact stats so the panel's summary counts read
            // with presence (caption over value) instead of four flat text rows.
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    mini_stat(ui, "Total", &self.fleet.nodes.len().to_string());
                    ui.add_space(theme::LG);
                    mini_stat(ui, "Running", &running.to_string());
                });
                ui.add_space(theme::SM);
                ui.horizontal(|ui| {
                    mini_stat(ui, "Stopped", &stopped.to_string());
                    ui.add_space(theme::LG);
                    mini_stat(ui, "Visible", &visible.to_string());
                });
            });
        });
    }
}
