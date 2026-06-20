use crate::{catalog::PluginId, types::StorageEngine};

pub(in crate::config) fn neo_cli_storage_engine(engine: StorageEngine) -> &'static str {
    match engine {
        StorageEngine::LevelDb => PluginId::LevelDbStore.into(),
        StorageEngine::RocksDb => PluginId::RocksDbStore.into(),
    }
}

impl From<PluginId> for &'static str {
    fn from(value: PluginId) -> Self {
        match value {
            PluginId::LevelDbStore => "LevelDBStore",
            PluginId::RocksDbStore => "RocksDBStore",
            PluginId::RpcServer => "RpcServer",
            PluginId::RestServer => "RestServer",
            PluginId::ApplicationLogs => "ApplicationLogs",
            PluginId::StateService => "StateService",
            PluginId::DBFTPlugin => "DBFTPlugin",
            PluginId::TokensTracker => "TokensTracker",
        }
    }
}
