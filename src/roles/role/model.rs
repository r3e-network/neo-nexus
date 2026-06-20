use std::fmt;

use crate::{
    catalog::PluginId,
    types::{NodeType, StorageEngine},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NodeRole {
    RpcApi,
    State,
    Indexer,
    Consensus,
    Observer,
}

impl NodeRole {
    pub const ALL: [Self; 5] = [
        Self::RpcApi,
        Self::State,
        Self::Indexer,
        Self::Consensus,
        Self::Observer,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::RpcApi => "RPC/API",
            Self::State => "State",
            Self::Indexer => "Indexer",
            Self::Consensus => "Consensus",
            Self::Observer => "Observer",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::RpcApi => "rpc-api",
            Self::State => "state",
            Self::Indexer => "indexer",
            Self::Consensus => "validator",
            Self::Observer => "observer",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::RpcApi => "Expose wallet, dApp, and operator JSON-RPC access.",
            Self::State => "Maintain state proof services for verification workflows.",
            Self::Indexer => "Collect application logs and token transfer indexes.",
            Self::Consensus => "Run private-network dBFT consensus duties.",
            Self::Observer => "Follow the chain and expose read-only operator access.",
        }
    }
}

impl fmt::Display for NodeRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RolePluginChange {
    pub plugin_id: PluginId,
    pub enabled: bool,
    pub reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RolePlan {
    pub role: NodeRole,
    pub node_type: NodeType,
    pub storage_engine: StorageEngine,
    pub plugin_changes: Vec<RolePluginChange>,
    pub notes: Vec<&'static str>,
}

impl RolePlan {
    pub fn change_count(&self) -> usize {
        self.plugin_changes.len()
    }
}
