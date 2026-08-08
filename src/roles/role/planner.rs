use crate::types::{NodeConfig, NodeType, StorageEngine};

use super::{neo_cli::neo_cli_plan, runtime_managed::runtime_managed_plan, NodeRole, RolePlan};

pub struct RolePlanner;

impl RolePlanner {
    pub fn plan(node: &NodeConfig, role: NodeRole) -> RolePlan {
        Self::plan_for(node.node_type, node.storage_engine, role)
    }

    pub fn plan_for(
        node_type: NodeType,
        storage_engine: StorageEngine,
        role: NodeRole,
    ) -> RolePlan {
        match node_type {
            NodeType::NeoCli => neo_cli_plan(storage_engine, role),
            NodeType::NeoGo => runtime_managed_plan(
                node_type,
                storage_engine,
                role,
                "neo-go configures every service in its generated YAML; there are no plugin assemblies to install.",
            ),
            NodeType::NeoRs => runtime_managed_plan(
                node_type,
                storage_engine,
                role,
                "neo-rs configures RPC, storage and consensus in its generated TOML.",
            ),
            NodeType::NeoXGeth => runtime_managed_plan(
                node_type,
                storage_engine,
                role,
                "Neo X Geth configures its RPC namespaces and peering in the generated TOML; consensus membership is decided on-chain.",
            ),
            NodeType::NeoXReth => runtime_managed_plan(
                node_type,
                storage_engine,
                role,
                "neox-rs takes its chain and ports as launch flags; validator mode also needs --validator.experimental on the public networks.",
            ),
        }
    }
}
