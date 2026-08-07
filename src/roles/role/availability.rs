//! Which duties each client can actually perform.
//!
//! Offering a role a client cannot run is worse than not offering it: the
//! operator configures it, the node starts, and nothing happens. The three
//! clients genuinely differ, and two differences are absolute:
//!
//! - **P2P Notary is neo-go only.** The C# node has no `P2PNotaryRequest`
//!   payload and no notary module, so no amount of configuration makes neo-cli
//!   serve notary requests. (The Notary *contract* is on-chain and visible to
//!   every client; running the *service* is not.)
//! - **neo-rs is unverified.** No part of this work established what neo-rs
//!   supports, and it must not inherit neo-go's answers by sharing a code path
//!   with it. Everything beyond the roles its existing TOML already models is
//!   reported as unverified rather than guessed at.

use crate::types::NodeType;

use super::NodeRole;

/// Whether a client can perform a duty, and why not when it cannot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleAvailability {
    Supported,
    /// The client cannot perform this duty at all.
    Unsupported(&'static str),
    /// Support has not been established. Distinct from `Unsupported`: this is a
    /// gap in what we know, not a fact about the client.
    Unverified(&'static str),
}

impl RoleAvailability {
    pub fn is_supported(self) -> bool {
        matches!(self, Self::Supported)
    }

    /// Operator-facing explanation, or `None` when the duty is available.
    pub fn reason(self) -> Option<&'static str> {
        match self {
            Self::Supported => None,
            Self::Unsupported(reason) | Self::Unverified(reason) => Some(reason),
        }
    }
}

/// The support matrix. Every cell is stated explicitly so a new role or client
/// cannot silently default to "supported".
pub fn role_availability(node_type: NodeType, role: NodeRole) -> RoleAvailability {
    match node_type {
        NodeType::NeoCli => neo_cli(role),
        NodeType::NeoGo => neo_go(role),
        NodeType::NeoRs => neo_rs(role),
    }
}

fn neo_cli(role: NodeRole) -> RoleAvailability {
    match role {
        NodeRole::RpcApi
        | NodeRole::State
        | NodeRole::Indexer
        | NodeRole::Consensus
        | NodeRole::Oracle
        | NodeRole::StateValidator
        | NodeRole::Observer => RoleAvailability::Supported,
        NodeRole::Notary => RoleAvailability::Unsupported(
            "neo-cli has no P2P notary service: the C# node implements neither the \
             P2PNotaryRequest payload nor a notary module.",
        ),
    }
}

fn neo_go(role: NodeRole) -> RoleAvailability {
    match role {
        NodeRole::RpcApi
        | NodeRole::State
        | NodeRole::Indexer
        | NodeRole::Consensus
        | NodeRole::Oracle
        | NodeRole::StateValidator
        | NodeRole::Notary
        | NodeRole::Observer => RoleAvailability::Supported,
    }
}

fn neo_rs(role: NodeRole) -> RoleAvailability {
    match role {
        // These are the postures the existing neo-rs TOML generator already
        // models, so they are known to be expressible.
        NodeRole::RpcApi | NodeRole::Consensus | NodeRole::Observer => RoleAvailability::Supported,
        NodeRole::State
        | NodeRole::Indexer
        | NodeRole::Oracle
        | NodeRole::StateValidator
        | NodeRole::Notary => RoleAvailability::Unverified(
            "neo-rs support for this duty has not been established; NeoNexus will not \
             generate configuration it cannot verify the node reads.",
        ),
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/roles/role/availability/tests.rs"]
mod tests;
