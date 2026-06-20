use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn selected_node(&self) -> Option<&NodeConfig> {
        self.selected_node
            .as_ref()
            .and_then(|id| self.nodes.iter().find(|node| &node.id == id))
    }

    pub(in crate::app) fn selected_node_index(&self) -> Option<usize> {
        self.selected_node
            .as_ref()
            .and_then(|id| self.nodes.iter().position(|node| &node.id == id))
    }

    pub(in crate::app) fn node_inventory_filter(&self) -> NodeInventoryFilter {
        NodeInventoryFilter::new(self.node_status_filter, self.node_query.as_str())
    }

    pub(in crate::app) fn filtered_inventory_nodes(&self) -> Vec<NodeConfig> {
        filter_nodes(&self.nodes, &self.node_inventory_filter())
    }

    pub(in crate::app) fn selected_filtered_node_index(
        &self,
        nodes: &[NodeConfig],
    ) -> Option<usize> {
        self.selected_node
            .as_ref()
            .and_then(|id| nodes.iter().position(|node| &node.id == id))
    }
}
