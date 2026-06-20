use anyhow::Result;

use crate::types::{Network, NewNode, NodeConfig, NodeType, StorageEngine};

use super::{
    conflicts::{detect_conflicts, PrivateNetworkConflict},
    template::PrivateNetworkTemplate,
};
use crate::roles::NodeRole;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateNetworkNodePlan {
    pub name: String,
    pub node_type: NodeType,
    pub role: NodeRole,
    pub network: Network,
    pub storage_engine: StorageEngine,
    pub rpc_port: u16,
    pub p2p_port: u16,
    pub ws_port: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateNetworkPlan {
    pub template: PrivateNetworkTemplate,
    pub node_type: NodeType,
    pub nodes: Vec<PrivateNetworkNodePlan>,
}

impl PrivateNetworkPlan {
    pub fn consensus_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|node| node.role == NodeRole::Consensus)
            .count()
    }

    pub fn to_new_nodes(&self, template_node: &NodeConfig) -> Result<Vec<NewNode>> {
        if template_node.node_type != self.node_type {
            anyhow::bail!(
                "template node runtime {} does not match private network runtime {}",
                template_node.node_type,
                self.node_type
            );
        }

        Ok(self
            .nodes
            .iter()
            .map(|node| NewNode {
                name: node.name.clone(),
                node_type: node.node_type,
                network: node.network,
                binary_path: template_node.binary_path.clone(),
                args: Vec::new(),
                runtime_version: template_node.runtime_version.clone(),
                storage_engine: node.storage_engine,
                rpc_port: node.rpc_port,
                p2p_port: node.p2p_port,
                ws_port: node.ws_port,
            })
            .collect())
    }

    pub fn conflicts_with(&self, existing_nodes: &[NodeConfig]) -> Vec<PrivateNetworkConflict> {
        detect_conflicts(self, existing_nodes)
    }
}
