use std::{fmt, str::FromStr};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PluginId {
    RpcServer,
    RestServer,
    ApplicationLogs,
    StateService,
    DBFTPlugin,
    TokensTracker,
    LevelDbStore,
    RocksDbStore,
}

impl fmt::Display for PluginId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::RpcServer => "RpcServer",
            Self::RestServer => "RestServer",
            Self::ApplicationLogs => "ApplicationLogs",
            Self::StateService => "StateService",
            Self::DBFTPlugin => "DBFTPlugin",
            Self::TokensTracker => "TokensTracker",
            Self::LevelDbStore => "LevelDBStore",
            Self::RocksDbStore => "RocksDBStore",
        })
    }
}

impl FromStr for PluginId {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "RpcServer" => Ok(Self::RpcServer),
            "RestServer" => Ok(Self::RestServer),
            "ApplicationLogs" => Ok(Self::ApplicationLogs),
            "StateService" => Ok(Self::StateService),
            "DBFTPlugin" => Ok(Self::DBFTPlugin),
            "TokensTracker" => Ok(Self::TokensTracker),
            "LevelDBStore" => Ok(Self::LevelDbStore),
            "RocksDBStore" => Ok(Self::RocksDbStore),
            other => anyhow::bail!("unsupported plugin id: {other}"),
        }
    }
}
