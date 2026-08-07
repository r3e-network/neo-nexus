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
    Oracle,
    StateValidator,
    Notary,
    Observer,
}

impl NodeRole {
    pub const ALL: [Self; 8] = [
        Self::RpcApi,
        Self::State,
        Self::Indexer,
        Self::Consensus,
        Self::Oracle,
        Self::StateValidator,
        Self::Notary,
        Self::Observer,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::RpcApi => "RPC/API",
            Self::State => "State",
            Self::Indexer => "Indexer",
            Self::Consensus => "Consensus",
            Self::Oracle => "Oracle",
            Self::StateValidator => "State Validator",
            Self::Notary => "Notary",
            Self::Observer => "Observer",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::RpcApi => "rpc-api",
            Self::State => "state",
            Self::Indexer => "indexer",
            Self::Consensus => "validator",
            Self::Oracle => "oracle",
            Self::StateValidator => "state-validator",
            Self::Notary => "notary",
            Self::Observer => "observer",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::RpcApi => "Expose wallet, dApp, and operator JSON-RPC access.",
            Self::State => "Maintain state proof services for verification workflows.",
            Self::Indexer => "Collect application logs and token transfer indexes.",
            Self::Consensus => "Take part in dBFT block production as a validator.",
            Self::Oracle => "Answer on-chain oracle requests over HTTPS and NeoFS.",
            Self::StateValidator => "Sign and publish state roots for cross-chain proofs.",
            Self::Notary => "Serve P2P notary requests for multi-signature assembly.",
            Self::Observer => "Follow the chain and expose read-only operator access.",
        }
    }

    /// Stable identifier used to persist the role against a node.
    pub fn persist_key(self) -> &'static str {
        self.slug()
    }

    pub fn from_persist_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|role| role.persist_key() == key)
    }

    /// The `RoleManagement` designation this duty requires before the node can
    /// perform it, if any. `RpcApi`, `Indexer` and `Observer` are purely local
    /// capabilities; the rest are committee-granted.
    pub fn designation(self) -> Option<ChainRole> {
        match self {
            Self::StateValidator => Some(ChainRole::StateValidator),
            Self::Oracle => Some(ChainRole::Oracle),
            Self::Notary => Some(ChainRole::P2PNotary),
            // A validator is elected by NEO holders through the committee vote,
            // not designated through RoleManagement.
            Self::Consensus | Self::RpcApi | Self::State | Self::Indexer | Self::Observer => None,
        }
    }
}

/// A role the `RoleManagement` native contract can designate a public key for.
///
/// The discriminants are the on-chain values passed to `designateAsRole` and
/// `getDesignatedByRole`; they are consensus-visible and must not be renumbered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChainRole {
    StateValidator = 4,
    Oracle = 8,
    NeoFSAlphabet = 16,
    P2PNotary = 32,
}

impl ChainRole {
    pub const ALL: [Self; 4] = [
        Self::StateValidator,
        Self::Oracle,
        Self::NeoFSAlphabet,
        Self::P2PNotary,
    ];

    /// The integer `RoleManagement` expects.
    pub fn on_chain_value(self) -> u8 {
        self as u8
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::StateValidator => "StateValidator",
            Self::Oracle => "Oracle",
            Self::NeoFSAlphabet => "NeoFSAlphabet",
            Self::P2PNotary => "P2PNotary",
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
