use super::*;

impl NeoNexusApp {
    pub(in crate::app::views::shell::inventory) fn render_inventory_nodes(
        &mut self,
        ui: &mut egui::Ui,
    ) {
        let nodes = self.filtered_inventory_nodes();

        if self.fleet.nodes.is_empty() {
            if empty_state_with_action(
                ui,
                "No nodes",
                "Define the first local Neo runtime to start operating.",
                Some("New Node"),
            ) {
                self.session.selected_view = crate::app::view::View::Nodes;
            }
            return;
        }

        if nodes.is_empty() {
            empty_state(
                ui,
                "No matching nodes",
                "No local nodes match the current filter.",
            );
            return;
        }

        let total_pages = page_count(nodes.len(), NODE_PAGE_SIZE);
        self.fleet.node_page = self.fleet.node_page.min(total_pages - 1);
        pagination_bar(ui, &mut self.fleet.node_page, total_pages, nodes.len());
        ui.add_space(theme::SM);

        let start = self.fleet.node_page * NODE_PAGE_SIZE;
        let visible: Vec<_> = nodes.iter().skip(start).take(NODE_PAGE_SIZE).collect();
        let mut next_selection = self.fleet.selected_node.clone();

        for row in 0..NODE_PAGE_SIZE {
            if let Some(node) = visible.get(row) {
                let selected = next_selection.as_deref() == Some(node.id.as_str());
                if node_row(ui, node, selected, true) {
                    next_selection = Some(node.id.clone());
                }
                ui.add_space(theme::XS);
            } else {
                // Reserve row height so the list stays evenly paced when a page
                // is only partially filled (matches compact node_row height).
                ui.add_space(theme::DensityMetrics::COMFORTABLE.list_row_compact);
            }
        }

        self.select_fleet_node(next_selection);
    }
}
