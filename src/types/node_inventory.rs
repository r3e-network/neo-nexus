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
        || text_matches(&node.status.to_string(), query)
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
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::types::{Network, NodeType, StorageEngine};

    #[test]
    fn node_inventory_filter_matches_operational_fields() {
        let nodes = [node("rpc-1", "RPC Alpha", NodeStatus::Running, 10332)];

        assert_ids(&nodes, NodeInventoryFilter::new(None, "alpha"), &["rpc-1"]);
        assert_ids(&nodes, NodeInventoryFilter::new(None, "neo-rs"), &["rpc-1"]);
        assert_ids(
            &nodes,
            NodeInventoryFilter::new(None, "testnet"),
            &["rpc-1"],
        );
        assert_ids(
            &nodes,
            NodeInventoryFilter::new(None, "running"),
            &["rpc-1"],
        );
        assert_ids(&nodes, NodeInventoryFilter::new(None, "10332"), &["rpc-1"]);
        assert_ids(&nodes, NodeInventoryFilter::new(None, "3.8.1"), &["rpc-1"]);
    }

    #[test]
    fn node_inventory_filter_combines_status_and_query() {
        let nodes = [
            node("rpc-1", "RPC Alpha", NodeStatus::Running, 10332),
            node("rpc-2", "RPC Beta", NodeStatus::Stopped, 20332),
            node("seed-1", "Seed Beta", NodeStatus::Running, 30332),
        ];
        let filter = NodeInventoryFilter::new(Some(NodeStatus::Running), "beta");

        assert_ids(&nodes, filter, &["seed-1"]);
    }

    fn assert_ids(nodes: &[NodeConfig], filter: NodeInventoryFilter, ids: &[&str]) {
        let filtered = filter_nodes(nodes, &filter);
        let actual = filtered
            .iter()
            .map(|node| node.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(actual.as_slice(), ids);
    }

    fn node(id: &str, name: &str, status: NodeStatus, rpc_port: u16) -> NodeConfig {
        NodeConfig {
            id: id.to_string(),
            name: name.to_string(),
            node_type: NodeType::NeoRs,
            network: Network::Testnet,
            binary_path: PathBuf::from("/opt/neo-node"),
            args: vec!["--config".to_string(), "config.toml".to_string()],
            runtime_version: "3.8.1".to_string(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port,
            p2p_port: rpc_port + 1,
            ws_port: Some(rpc_port + 2),
            status,
            pid: None,
        }
    }
}
