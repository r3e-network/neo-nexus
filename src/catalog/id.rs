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
    OracleService,
    TokensTracker,
    LevelDbStore,
    RocksDbStore,
    SignClient,
}

impl fmt::Display for PluginId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::RpcServer => "RpcServer",
            Self::RestServer => "RestServer",
            Self::ApplicationLogs => "ApplicationLogs",
            Self::StateService => "StateService",
            Self::DBFTPlugin => "DBFTPlugin",
            Self::OracleService => "OracleService",
            Self::TokensTracker => "TokensTracker",
            Self::LevelDbStore => "LevelDBStore",
            Self::RocksDbStore => "RocksDBStore",
            Self::SignClient => "SignClient",
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
            "OracleService" => Ok(Self::OracleService),
            "TokensTracker" => Ok(Self::TokensTracker),
            "LevelDBStore" => Ok(Self::LevelDbStore),
            "RocksDBStore" => Ok(Self::RocksDbStore),
            "SignClient" => Ok(Self::SignClient),
            other => anyhow::bail!("unsupported plugin id: {other}"),
        }
    }
}
