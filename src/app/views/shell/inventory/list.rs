use super::*;

impl NeoNexusApp {
    pub(in crate::app::views::shell::inventory) fn render_inventory_nodes(
        &mut self,
        ui: &mut egui::Ui,
    ) {
        let nodes = self.filtered_inventory_nodes();

        if self.nodes.is_empty() {
            empty_state(
                ui,
                "No nodes",
                "Use New Node to define the first local runtime.",
            );
            return;
        }

        if nodes.is_empty() {
            empty_state(
                ui,
                "No matching nodes",
                "No local nodes match current filter.",
            );
            return;
        }

        let total_pages = page_count(nodes.len(), NODE_PAGE_SIZE);
        self.node_page = self.node_page.min(total_pages - 1);
        pagination_bar(ui, &mut self.node_page, total_pages, nodes.len());
        ui.separator();

        let start = self.node_page * NODE_PAGE_SIZE;
        let visible: Vec<_> = nodes.iter().skip(start).take(NODE_PAGE_SIZE).collect();
        let mut next_selection = self.selected_node.clone();

        for row in 0..NODE_PAGE_SIZE {
            if let Some(node) = visible.get(row) {
                let selected = next_selection.as_deref() == Some(node.id.as_str());
                let label = format!(
                    "{}  {}  :{}",
                    truncate_middle(&node.name, 18),
                    node.network,
                    node.rpc_port
                );
                let response = ui.add_sized(
                    [ui.available_width(), 32.0],
                    egui::Button::new(label).selected(selected),
                );

                if response.clicked() {
                    next_selection = Some(node.id.clone());
                }

                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(node.node_type.to_string()).color(muted_text()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new(node.status.label())
                                .color(status_color(node.status))
                                .strong(),
                        );
                    });
                });
            } else {
                ui.add_space(52.0);
            }
        }

        if self.selected_node != next_selection {
            self.selected_node = next_selection;
            self.selected_plugin = None;
            self.plugin_page = 0;
            self.config_page = 0;
            self.log_page = 0;
        }
    }
}
