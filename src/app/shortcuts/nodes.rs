use crate::app::domain::NodeConfig;

use super::{NeoNexusApp, NODE_PAGE_SIZE};

pub(super) const NODE_SHORTCUT_PAGE_SIZE: usize = NODE_PAGE_SIZE;

impl NeoNexusApp {
    pub(super) fn shift_selected_node(&mut self, delta: isize) {
        let visible_nodes = self.filtered_inventory_nodes();
        let Some(next_index) = shifted_node_index(
            self.selected_filtered_node_index(&visible_nodes),
            visible_nodes.len(),
            delta,
        ) else {
            self.session.notice = Some("No nodes to select".to_string());
            return;
        };
        self.select_node_index_from_visible(&visible_nodes, next_index);
    }

    pub(super) fn select_node_index(&mut self, index: usize) {
        let visible_nodes = self.filtered_inventory_nodes();
        self.select_node_index_from_visible(&visible_nodes, index);
    }

    pub(in crate::app) fn visible_node_count(&self) -> usize {
        self.filtered_inventory_nodes().len()
    }

    fn select_node_index_from_visible(&mut self, visible_nodes: &[NodeConfig], index: usize) {
        let Some(node) = visible_nodes.get(index) else {
            self.session.notice = Some("No nodes to select".to_string());
            return;
        };
        self.fleet.selected_node = Some(node.id.clone());
        self.fleet.node_page = index / NODE_SHORTCUT_PAGE_SIZE;
        self.selected_plugin = None;
        self.plugin_page = 0;
        self.config_page = 0;
    }
}

pub(super) fn shifted_node_index(
    current_index: Option<usize>,
    node_count: usize,
    delta: isize,
) -> Option<usize> {
    if node_count == 0 {
        return None;
    }

    let Some(current) = current_index else {
        return Some(if delta < 0 { node_count - 1 } else { 0 });
    };
    let next = current.saturating_add_signed(delta);
    Some(next.min(node_count - 1))
}
