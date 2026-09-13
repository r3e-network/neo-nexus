use std::{fmt, path::PathBuf, str::FromStr};

use anyhow::Result;
use serde::Serialize;

use super::{ChainFamily, StorageEngine};
use crate::config::ConfigFormat;

/// Trait defining type-agnostic interface for node runtime variants.
///
/// This abstraction enables generic operations across different Neo node
/// implementations (neo-cli, neo-go, neo-rs, neox-geth, neox-rs) without
/// requiring callers to know type-specific details.
pub trait NodeTypeTraits {
    /// Returns configuration file format (JSON, YAML, or TOML)
    fn config_format(&self) -> ConfigFormat;

    /// Returns expected config path relative to working directory
    fn config_path(&self) -> PathBuf;

    /// Returns plugin directory path if applicable (None for types without plugins)
    fn plugin_directory(&self) -> Option<PathBuf>;

    /// Returns true if type supports dynamic plugins/extensions
    fn supports_plugins(&self) -> bool;

    /// Returns the default runtime binary name for this type
    fn default_binary_name(&self) -> &str;
}

impl NodeTypeTraits for NodeType {
    fn config_format(&self) -> ConfigFormat {
        match self {
            Self::NeoCli => ConfigFormat::Json,
            Self::NeoGo => ConfigFormat::Yaml,
            Self::NeoRs | Self::NeoXGeth | Self::NeoXReth => ConfigFormat::Json,
        }
    }

    fn config_path(&self) -> PathBuf {
        match self {
            Self::NeoCli => PathBuf::from("config.json"),
            Self::NeoGo => PathBuf::from("config/config.yml"),
            Self::NeoRs | Self::NeoXGeth | Self::NeoXReth => PathBuf::from("config/config.json"),
        }
    }

    fn plugin_directory(&self) -> Option<PathBuf> {
        match self {
            Self::NeoCli => Some(PathBuf::from("Plugins")),
            Self::NeoGo | Self::NeoRs | Self::NeoXGeth | Self::NeoXReth => None,
        }
    }

    fn supports_plugins(&self) -> bool {
        *self == Self::NeoCli
    }

    fn default_binary_name(&self) -> &str {
        match self {
            Self::NeoCli => "neo-cli.exe",
            Self::NeoGo => "neo-go",
            Self::NeoRs => "neo-node",
            Self::NeoXGeth => "neox-geth",
            Self::NeoXReth => "neox-rs",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NodeType {
    NeoCli,
    NeoGo,
    NeoRs,
    /// Neo X reference client: a go-ethereum fork with dBFT finality.
    ///
    /// Renamed explicitly: `rename_all = "kebab-case"` would derive
    /// `neo-x-geth`, which `FromStr` rejects, so a script reading `node_type`
    /// out of a `--*-json` action and feeding it back to another invocation
    /// would break on Neo X and only on Neo X.
    #[serde(rename = "neox-geth")]
    NeoXGeth,
    /// Independent Rust Neo X node built on Reth.
    #[serde(rename = "neox-rs")]
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

    /// What this client actually stores its chain in, for display.
    ///
    /// For Neo N3 clients this is the configured engine, because it is a real
    /// choice. Neither Neo X client offers one — Geth keeps its own Pebble
    /// store and neox-rs keeps Reth's MDBX — so their `storage_engine` field
    /// is a placeholder, and printing it would name a database the node never
    /// opens.
    pub fn storage_label(self, configured: StorageEngine) -> String {
        match self {
            Self::NeoXGeth => "Pebble (built in)".to_string(),
            Self::NeoXReth => "MDBX (built in)".to_string(),
            Self::NeoCli | Self::NeoGo | Self::NeoRs => configured.to_string(),
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

    /// Infers node type from a binary path, filename, or text identifier.
    ///
    /// Matches common executable filenames and binary stems across platforms
    /// (e.g. `neox-geth`, `geth`, `neo-node`, `neo-cli`, `neo-go`, `reth`, `neox-rs`).
    pub fn infer_from_str(s: &str) -> Option<Self> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return None;
        }
        if let Ok(direct) = trimmed.parse::<Self>() {
            return Some(direct);
        }
        let normalized = trimmed.replace('\\', "/");
        let filename = normalized.rsplit('/').next().unwrap_or(trimmed);
        let lower = filename.to_ascii_lowercase();
        let stem = lower.strip_suffix(".exe").unwrap_or(&lower);
        let stem = stem.strip_suffix(".dll").unwrap_or(stem);

        match stem {
            "neo-cli" => Some(Self::NeoCli),
            "neo-go" => Some(Self::NeoGo),
            "neo-node" | "neo-rs" => Some(Self::NeoRs),
            "neox-geth" | "geth" => Some(Self::NeoXGeth),
            "neox-rs" | "reth" | "neox-reth" => Some(Self::NeoXReth),
            _ => {
                if stem.contains("neox-geth") || (stem.contains("geth") && !stem.contains("reth")) {
                    Some(Self::NeoXGeth)
                } else if stem.contains("neox-rs") || stem.contains("reth") {
                    Some(Self::NeoXReth)
                } else if stem.contains("neo-node") {
                    Some(Self::NeoRs)
                } else if stem.contains("neo-go") {
                    Some(Self::NeoGo)
                } else if stem.contains("neo-cli") {
                    Some(Self::NeoCli)
                } else {
                    None
                }
            }
        }
    }

    /// Infers node type from a file path.
    pub fn infer_from_path(path: &std::path::Path) -> Option<Self> {
        path.file_name()
            .and_then(|n| n.to_str())
            .and_then(Self::infer_from_str)
            .or_else(|| path.to_str().and_then(Self::infer_from_str))
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

#[cfg(test)]
#[path = "../../tests/unit/types/node_type/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "../../tests/unit/types/node_type/traits_tests.rs"]
mod traits_tests;

