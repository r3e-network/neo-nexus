use std::{fmt, str::FromStr};

use anyhow::Result;
use serde::Serialize;

use super::StorageEngine;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NodeType {
    NeoCli,
    NeoGo,
    NeoRs,
}

impl NodeType {
    pub const ALL: [Self; 3] = [Self::NeoCli, Self::NeoGo, Self::NeoRs];

    pub fn default_storage_engine(self) -> StorageEngine {
        match self {
            Self::NeoCli | Self::NeoRs => StorageEngine::RocksDb,
            Self::NeoGo => StorageEngine::LevelDb,
        }
    }

    pub fn supports_storage_engine(self, storage_engine: StorageEngine) -> bool {
        match self {
            Self::NeoCli => matches!(
                storage_engine,
                StorageEngine::LevelDb | StorageEngine::RocksDb
            ),
            Self::NeoGo => storage_engine == StorageEngine::LevelDb,
            Self::NeoRs => storage_engine == StorageEngine::RocksDb,
        }
    }
}

impl fmt::Display for NodeType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NeoCli => "neo-cli",
            Self::NeoGo => "neo-go",
            Self::NeoRs => "neo-rs",
        })
    }
}

impl FromStr for NodeType {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "neo-cli" => Ok(Self::NeoCli),
            "neo-go" => Ok(Self::NeoGo),
            "neo-rs" => Ok(Self::NeoRs),
            other => anyhow::bail!("unsupported node type: {other}"),
        }
    }
}
