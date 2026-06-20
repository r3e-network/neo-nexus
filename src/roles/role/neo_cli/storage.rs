use crate::{catalog::PluginId, roles::RolePluginChange, types::StorageEngine};

use super::changes::{disable, enable};

pub(super) fn storage_alignment(storage_engine: StorageEngine) -> Vec<RolePluginChange> {
    match storage_engine {
        StorageEngine::LevelDb => vec![
            enable(
                PluginId::LevelDbStore,
                "Storage plugin matches the selected LevelDB engine.",
            ),
            disable(
                PluginId::RocksDbStore,
                "Only one neo-cli storage plugin should be active.",
            ),
        ],
        StorageEngine::RocksDb => vec![
            enable(
                PluginId::RocksDbStore,
                "Storage plugin matches the selected RocksDB engine.",
            ),
            disable(
                PluginId::LevelDbStore,
                "Only one neo-cli storage plugin should be active.",
            ),
        ],
    }
}
