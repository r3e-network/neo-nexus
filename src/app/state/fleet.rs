//! Fleet inventory UI state: node list, selection, draft editor, and shared
//! filter/paging for Home, Inventory, and node workspaces.

use crate::app::{
    domain::{NodeConfig, NodeStatus},
    draft::NodeDraft,
};

#[derive(Debug, Default)]
pub(in crate::app) struct FleetUi {
    pub(in crate::app) nodes: Vec<NodeConfig>,
    pub(in crate::app) selected_node: Option<String>,
    pub(in crate::app) draft: NodeDraft,
    pub(in crate::app) pending_delete_node: Option<String>,
    pub(in crate::app) overview_fleet_page: usize,
    pub(in crate::app) node_page: usize,
    pub(in crate::app) node_query: String,
    pub(in crate::app) node_status_filter: Option<NodeStatus>,
}

impl FleetUi {
    pub(in crate::app) fn reset_paging(&mut self) {
        self.node_page = 0;
        self.overview_fleet_page = 0;
    }

    pub(in crate::app) fn set_status_filter(&mut self, status: Option<NodeStatus>) {
        self.node_status_filter = status;
        self.reset_paging();
    }

    pub(in crate::app) fn select_node(&mut self, id: Option<String>) {
        self.selected_node = id;
    }

    pub(in crate::app) fn running_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|node| node.status.is_running())
            .count()
    }
}
