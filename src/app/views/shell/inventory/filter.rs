use super::*;

impl NeoNexusApp {
    pub(in crate::app::views::shell::inventory) fn render_inventory_filter(
        &mut self,
        ui: &mut egui::Ui,
    ) {
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.label(egui::RichText::new("Status").color(muted_text()));
            status_button(self, ui, "All", None);
            for status in NodeStatus::ALL {
                status_button(self, ui, status.label(), Some(status));
            }
        });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            let response = ui.add_sized(
                [(ui.available_width() - 10.0).max(120.0), 24.0],
                egui::TextEdit::singleline(&mut self.node_query).hint_text("Search"),
            );
            if response.changed() {
                self.node_page = 0;
                self.overview_fleet_page = 0;
            }
        });
    }
}

fn status_button(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    label: &str,
    status: Option<NodeStatus>,
) {
    if ui
        .selectable_label(app.node_status_filter == status, label)
        .clicked()
    {
        app.node_status_filter = status;
        app.node_page = 0;
        app.overview_fleet_page = 0;
    }
}
