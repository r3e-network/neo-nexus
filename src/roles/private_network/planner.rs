use crate::{
    roles::NodeRole,
    types::{Network, NodeType, StorageEngine},
};

use super::{
    plan::{PrivateNetworkNodePlan, PrivateNetworkPlan},
    template::PrivateNetworkTemplate,
};

pub struct PrivateNetworkPlanner;

impl PrivateNetworkPlanner {
    pub fn plan(template: PrivateNetworkTemplate, node_type: NodeType) -> PrivateNetworkPlan {
        let roles = match template {
            PrivateNetworkTemplate::SingleValidator => vec![NodeRole::Consensus],
            PrivateNetworkTemplate::FourValidators => vec![
                NodeRole::Consensus,
                NodeRole::Consensus,
                NodeRole::Consensus,
                NodeRole::Consensus,
            ],
            PrivateNetworkTemplate::SevenNodeLab => vec![
                NodeRole::Consensus,
                NodeRole::Consensus,
                NodeRole::Consensus,
                NodeRole::Consensus,
                NodeRole::RpcApi,
                NodeRole::State,
                NodeRole::Indexer,
            ],
        };

        let nodes = roles
            .into_iter()
            .enumerate()
            .map(|(index, role)| private_node_plan(index, node_type, role))
            .collect();

        PrivateNetworkPlan {
            template,
            node_type,
            nodes,
        }
    }
}

fn private_node_plan(index: usize, node_type: NodeType, role: NodeRole) -> PrivateNetworkNodePlan {
    let block = index as u16 * 10;
    let role_index = index + 1;
    PrivateNetworkNodePlan {
        name: format!("{node_type}-{}-{role_index}", role.slug()),
        node_type,
        role,
        network: Network::Private,
        storage_engine: default_storage(node_type),
        rpc_port: 30_332 + block,
        p2p_port: 30_333 + block,
        ws_port: Some(30_334 + block),
    }
}

fn default_storage(node_type: NodeType) -> StorageEngine {
    node_type.default_storage_engine()
}
