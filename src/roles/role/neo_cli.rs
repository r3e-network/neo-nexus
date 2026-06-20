use crate::types::{NodeType, StorageEngine};

use super::{NodeRole, RolePlan};

mod changes;
mod presets;
mod storage;

use self::{presets::role_plugin_changes, storage::storage_alignment};

pub(super) fn neo_cli_plan(storage_engine: StorageEngine, role: NodeRole) -> RolePlan {
    let mut plugin_changes = role_plugin_changes(role);
    plugin_changes.extend(storage_alignment(storage_engine));

    RolePlan {
        role,
        node_type: NodeType::NeoCli,
        storage_engine,
        plugin_changes,
        notes: vec![
            role.description(),
            "neo-cli role presets are applied through plugin state and exported JSON config.",
        ],
    }
}
