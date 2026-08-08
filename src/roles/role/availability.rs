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
//! - **neo-rs serves state but does not sign it, and cannot be an oracle.**
//!   Its daemon config (`neo-node/src/node/config.rs` in r3e-network/neo-rs)
//!   has a `[state_service]` section for tracking and serving state roots, but
//!   the only signing key it accepts anywhere is `[consensus] private_key_hex`
//!   — there is no state-root witness path. And while the workspace contains a
//!   `neo-oracle-service` crate, the daemon uses it only to recognise oracle
//!   response transactions when validating a block proposal; no `[oracle]`
//!   config section exists, so an operator cannot run one.
//! - **Neo X has no designated duties at all.** Oracle, StateValidator and
//!   P2PNotary are `RoleManagement` designations on a Neo N3 native contract.
//!   Neo X is an EVM sidechain: that contract does not exist there, and neither
//!   do the services behind it.

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
        NodeType::NeoXGeth | NodeType::NeoXReth => neox(role),
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
        // Each of these has a section in the daemon's TOML config: [rpc],
        // [consensus], [state_service], and the [indexer] / [application_logs]
        // / [tokens_tracker] trio.
        NodeRole::RpcApi
        | NodeRole::Consensus
        | NodeRole::State
        | NodeRole::Indexer
        | NodeRole::Observer => RoleAvailability::Supported,
        NodeRole::StateValidator => RoleAvailability::Unsupported(
            "neo-rs tracks and serves state roots but does not sign them: the only signing \
             key its daemon accepts is the consensus key.",
        ),
        NodeRole::Oracle => RoleAvailability::Unsupported(
            "neo-rs has no oracle configuration: its daemon recognises oracle response \
             transactions when validating blocks, but cannot answer oracle requests.",
        ),
        NodeRole::Notary => RoleAvailability::Unsupported("neo-rs has no P2P notary service."),
    }
}

/// Both Neo X clients answer the same way: the differences between Geth and
/// neox-rs are in how a duty is switched on, not in which duties exist.
fn neox(role: NodeRole) -> RoleAvailability {
    match role {
        // The EVM JSON-RPC surface covers all four: `eth_getProof` serves state
        // proofs and `eth_getLogs` serves the receipt index, so neither needs a
        // service of its own the way Neo N3 does.
        NodeRole::RpcApi | NodeRole::State | NodeRole::Indexer | NodeRole::Observer => {
            RoleAvailability::Supported
        }
        // Both clients ship dBFT block production, keyed by an ECDSA signing
        // key and a DKG share. Membership is decided on-chain, not by config.
        NodeRole::Consensus => RoleAvailability::Supported,
        NodeRole::Oracle => RoleAvailability::Unsupported(
            "Neo X has no oracle service: the Oracle designation lives on the Neo N3 \
             RoleManagement native contract, which an EVM sidechain does not have.",
        ),
        NodeRole::StateValidator => RoleAvailability::Unsupported(
            "Neo X needs no state-root signers: every EVM block header already commits to the \
             state root, so there is nothing separate to sign or publish.",
        ),
        NodeRole::Notary => RoleAvailability::Unsupported(
            "Neo X has no P2P notary service: the P2PNotaryRequest payload is a Neo N3 \
             protocol message with no EVM equivalent.",
        ),
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/roles/role/availability/tests.rs"]
mod tests;
