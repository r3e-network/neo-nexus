use std::{fmt, str::FromStr};

use anyhow::Result;
use serde::Serialize;

use super::{ChainFamily, StorageEngine};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NodeType {
    NeoCli,
    NeoGo,
    NeoRs,
    /// Neo X reference client: a go-ethereum fork with dBFT finality.
    NeoXGeth,
    /// Independent Rust Neo X node built on Reth.
    NeoXReth,
}

impl NodeType {
    pub const ALL: [Self; 5] = [
        Self::NeoCli,
        Self::NeoGo,
        Self::NeoRs,
        Self::NeoXGeth,
        Self::NeoXReth,
    ];

    /// Which chain this client joins. Derived, not stored: no client speaks
    /// both, so a node's family is a fact about its binary.
    pub fn family(self) -> ChainFamily {
        match self {
            Self::NeoCli | Self::NeoGo | Self::NeoRs => ChainFamily::NeoN3,
            Self::NeoXGeth | Self::NeoXReth => ChainFamily::NeoX,
        }
    }

    pub fn default_storage_engine(self) -> StorageEngine {
        match self {
            Self::NeoCli | Self::NeoRs => StorageEngine::RocksDb,
            Self::NeoGo => StorageEngine::LevelDb,
            // Both Neo X clients keep their own embedded store — geth uses
            // Pebble, neox-rs uses Reth's MDBX — and neither is selectable, so
            // the field is not an operator choice on this family.
            Self::NeoXGeth | Self::NeoXReth => StorageEngine::RocksDb,
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
            Self::NeoXGeth | Self::NeoXReth => storage_engine == StorageEngine::RocksDb,
        }
    }
}

impl fmt::Display for NodeType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NeoCli => "neo-cli",
            Self::NeoGo => "neo-go",
            Self::NeoRs => "neo-rs",
            Self::NeoXGeth => "neox-geth",
            Self::NeoXReth => "neox-rs",
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
            "neox-geth" => Ok(Self::NeoXGeth),
            "neox-rs" => Ok(Self::NeoXReth),
            other => anyhow::bail!("unsupported node type: {other}"),
        }
    }
}
