use crate::types::{NodeType, StorageEngine};

use super::{NodeRole, RolePlan};

pub(super) fn runtime_managed_plan(
    node_type: NodeType,
    storage_engine: StorageEngine,
    role: NodeRole,
    runtime_note: &'static str,
) -> RolePlan {
    let mut notes = vec![role.description(), runtime_note];
    notes.push(match role {
        NodeRole::RpcApi | NodeRole::Observer => {
            "RPC posture is represented in managed configuration, not plugin state."
        }
        NodeRole::State => {
            "State service parity depends on runtime support and managed configuration."
        }
        NodeRole::Indexer => {
            "Indexer packages are runtime-specific and are not modeled as neo-cli plugins."
        }
        NodeRole::Consensus => {
            "Consensus activation requires private-network keys and runtime configuration."
        }
    });

    RolePlan {
        role,
        node_type,
        storage_engine,
        plugin_changes: Vec::new(),
        notes,
    }
}
