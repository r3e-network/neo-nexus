use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn reload_nodes(&mut self) {
        match self.repository.list_nodes() {
            Ok(nodes) => {
                self.nodes = nodes;
                self.ensure_valid_selection();
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn ensure_valid_selection(&mut self) {
        let selected_exists = self
            .selected_node
            .as_ref()
            .is_some_and(|id| self.nodes.iter().any(|node| &node.id == id));

        if !selected_exists {
            self.selected_node = self.nodes.first().map(|node| node.id.clone());
            self.selected_plugin = None;
            self.plugin_page = 0;
            self.config_page = 0;
            self.log_page = 0;
            self.event_page = 0;
            self.selected_event = None;
        }

        self.node_page = clamp_page(
            self.node_page,
            self.filtered_inventory_nodes().len(),
            NODE_PAGE_SIZE,
        );
        self.overview_fleet_page = clamp_page(
            self.overview_fleet_page,
            self.filtered_inventory_nodes().len(),
            OVERVIEW_FLEET_PAGE_SIZE,
        );
    }
}
