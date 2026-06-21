use super::*;

impl NeoNexusApp {
    pub(in crate::app::views::shell::inventory) fn render_inventory_metrics(
        &self,
        ui: &mut egui::Ui,
    ) {
        let running = self
            .nodes
            .iter()
            .filter(|node| node.status.is_running())
            .count();
        let stopped = self
            .nodes
            .iter()
            .filter(|node| node.status.is_stopped())
            .count();
        let filter = self.node_inventory_filter();
        let visible = if filter.is_empty() {
            self.nodes.len()
        } else {
            self.filtered_inventory_nodes().len()
        };

        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.vertical(|ui| {
                fact(ui, "Total", &self.nodes.len().to_string());
                fact(ui, "Running", &running.to_string());
                fact(ui, "Stopped", &stopped.to_string());
                fact(ui, "Visible", &visible.to_string());
            });
        });
    }
}
