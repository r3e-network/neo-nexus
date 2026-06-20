use std::{fmt, str::FromStr};

use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageEngine {
    LevelDb,
    RocksDb,
}

impl StorageEngine {
    pub const ALL: [Self; 2] = [Self::LevelDb, Self::RocksDb];
}

impl fmt::Display for StorageEngine {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::LevelDb => "leveldb",
            Self::RocksDb => "rocksdb",
        })
    }
}

impl FromStr for StorageEngine {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "leveldb" => Ok(Self::LevelDb),
            "rocksdb" => Ok(Self::RocksDb),
            other => anyhow::bail!("unsupported storage engine: {other}"),
        }
    }
}
