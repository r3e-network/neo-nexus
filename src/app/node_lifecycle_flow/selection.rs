use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn selected_node(&self) -> Option<&NodeConfig> {
        self.fleet.selected_node
            .as_ref()
            .and_then(|id| self.fleet.nodes.iter().find(|node| &node.id == id))
    }

    pub(in crate::app) fn selected_node_index(&self) -> Option<usize> {
        self.fleet.selected_node
            .as_ref()
            .and_then(|id| self.fleet.nodes.iter().position(|node| &node.id == id))
    }

    pub(in crate::app) fn node_inventory_filter(&self) -> NodeInventoryFilter {
        NodeInventoryFilter::new(self.fleet.node_status_filter, self.fleet.node_query.as_str())
    }

    pub(in crate::app) fn filtered_inventory_nodes(&self) -> Vec<NodeConfig> {
        filter_nodes(&self.fleet.nodes, &self.node_inventory_filter())
    }

    pub(in crate::app) fn selected_filtered_node_index(
        &self,
        nodes: &[NodeConfig],
    ) -> Option<usize> {
        self.fleet.selected_node
            .as_ref()
            .and_then(|id| nodes.iter().position(|node| &node.id == id))
    }
}
