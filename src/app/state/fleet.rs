//! Fleet-facing UI helpers: shared paging/filter/selection mutations used by
//! Home, Inventory, and fleet snapshot surfaces. Fields still live on
//! `NeoNexusApp`; this module is the incremental home for list-unit behaviour
//! so those surfaces stop re-implementing the same reset/select logic.

use crate::app::{domain::NodeStatus, NeoNexusApp};

impl NeoNexusApp {
    /// Reset paging when a fleet-wide filter changes so operators never land on
    /// an empty page after narrowing the list.
    pub(in crate::app) fn reset_fleet_paging(&mut self) {
        self.node_page = 0;
        self.overview_fleet_page = 0;
    }

    pub(in crate::app) fn set_fleet_status_filter(&mut self, status: Option<NodeStatus>) {
        self.node_status_filter = status;
        self.reset_fleet_paging();
    }

    pub(in crate::app) fn select_fleet_node(&mut self, id: Option<String>) {
        if self.selected_node == id {
            return;
        }
        self.selected_node = id;
        self.selected_plugin = None;
        self.plugin_page = 0;
        self.config_page = 0;
        self.log_page = 0;
    }

    pub(in crate::app) fn running_node_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|node| node.status.is_running())
            .count()
    }
}
