use super::*;

impl NeoNexusApp {
    pub(in crate::app::views::shell::inventory) fn render_inventory_filter(
        &mut self,
        ui: &mut egui::Ui,
    ) {
        ui.horizontal(|ui| {
            ui.label(theme::muted_body("Status"));
            chip_pill(ui, |ui| {
                if filter_chip(ui, "All", self.fleet.node_status_filter.is_none()) {
                    self.set_fleet_status_filter(None);
                }
                for status in NodeStatus::ALL {
                    if filter_chip(
                        ui,
                        status.label(),
                        self.fleet.node_status_filter == Some(status),
                    ) {
                        self.set_fleet_status_filter(Some(status));
                    }
                }
            });
        });
        ui.add_space(theme::XS);
        if filter_bar(ui, &mut self.fleet.node_query, "Search nodes") {
            self.reset_fleet_paging();
        }
    }
}
