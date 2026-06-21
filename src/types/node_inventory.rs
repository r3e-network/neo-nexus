use super::{NodeConfig, NodeStatus};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NodeInventoryFilter {
    pub status: Option<NodeStatus>,
    pub query: String,
}

impl NodeInventoryFilter {
    pub fn new(status: Option<NodeStatus>, query: impl Into<String>) -> Self {
        Self {
            status,
            query: query.into(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.status.is_none() && self.query.trim().is_empty()
    }
}

pub fn filter_nodes(nodes: &[NodeConfig], filter: &NodeInventoryFilter) -> Vec<NodeConfig> {
    let query = filter.query.trim().to_lowercase();
    nodes
        .iter()
        .filter(|node| filter.status.is_none_or(|status| node.status == status))
        .filter(|node| query.is_empty() || node_matches(node, &query))
        .cloned()
        .collect()
}

fn node_matches(node: &NodeConfig, query: &str) -> bool {
    text_matches(node.id.as_str(), query)
        || text_matches(node.name.as_str(), query)
        || text_matches(&node.node_type.to_string(), query)
        || text_matches(&node.network.to_string(), query)
        || text_matches(node.status.label(), query)
        || text_matches(&node.storage_engine.to_string(), query)
        || text_matches(&node.runtime_version, query)
        || text_matches(&node.binary_path.display().to_string(), query)
        || text_matches(&node.rpc_port.to_string(), query)
        || text_matches(&node.p2p_port.to_string(), query)
        || node
            .ws_port
            .is_some_and(|port| text_matches(&port.to_string(), query))
}

fn text_matches(value: &str, query: &str) -> bool {
    value.to_lowercase().contains(query)
}

#[cfg(test)]
#[path = "../../tests/unit/types/node_inventory/tests.rs"]
mod tests;
