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
                "neo-go exposes RPC and core runtime posture through generated YAML configuration.",
            ),
            NodeType::NeoRs => runtime_managed_plan(
                node_type,
                storage_engine,
                role,
                "neo-rs exposes RPC, storage, and consensus posture through generated TOML configuration.",
            ),
        }
    }
}
