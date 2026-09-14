//! Whether a duty can actually be launched, not merely whether a client has it.
//!
//! `role_availability` describes what a **client** can do. It said Neo X
//! supports Consensus — which is true of geth and neox-reth, both of which ship
//! dBFT block production — and `/roles` let an operator apply it. Launch then
//! refused every possible way: with no key, because a signing duty requires one;
//! with a local wallet, because "NeoX key material cannot consume a Neo N3 NEP-6
//! signer profile"; with a local signer or the remote service, because "only
//! neo-cli consensus is supported". No `--validator` or `--miner` flag is
//! emitted anywhere either.
//!
//! So the matrix offered a duty that could not be reached from any path in this
//! product. Worse, `role_availability` is consulted by **nothing on the launch
//! path** — the module claims it is — so an unsupported duty like Oracle on
//! neo-rs was silently accepted and simply never took effect.
//!
//! This is the second gate: which duties this build can carry all the way to a
//! running process. It lives beside the availability matrix so the two are read
//! together, and the launch path's own signer checks defer to it rather than
//! restating it.

use crate::types::NodeType;

use super::model::NodeRole;

/// Why a duty a client supports still cannot be launched here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchSupport {
    /// NeoNexus can configure this duty and start the node performing it.
    Launchable,
    /// The client can do it; this product cannot set it up.
    NotImplemented(&'static str),
}

impl LaunchSupport {
    pub fn is_launchable(self) -> bool {
        matches!(self, Self::Launchable)
    }

    pub fn reason(self) -> Option<&'static str> {
        match self {
            Self::Launchable => None,
            Self::NotImplemented(reason) => Some(reason),
        }
    }
}

/// Whether this build can launch `node_type` performing `role`.
///
/// Non-signing duties are configuration only: a plugin or a service section in
/// the generated config, which every client path can write. The gap is entirely
/// in the signing duties, where a key has to reach a client in a form that
/// client accepts.
pub fn launch_support(node_type: NodeType, role: NodeRole) -> LaunchSupport {
    if !role.requires_signer() {
        return LaunchSupport::Launchable;
    }
    match node_type {
        // neo-go takes a wallet path and a password in each service section, so
        // every signing duty it supports is reachable.
        NodeType::NeoGo => LaunchSupport::Launchable,
        // neo-cli opens one wallet for all of its signing services.
        NodeType::NeoCli => LaunchSupport::Launchable,
        NodeType::NeoRs => LaunchSupport::NotImplemented(
            "neo-rs takes its consensus key as a plaintext hex string in its config, which this \
             workspace will not write. There is no NEP-6 path into its daemon.",
        ),
        NodeType::NeoXGeth | NodeType::NeoXReth => LaunchSupport::NotImplemented(
            "Neo X validators sign with secp256k1 key material and a DKG share. NeoNexus holds \
             Neo N3 NEP-6 wallets and speaks the Neo SecureSign protocol, neither of which a \
             Neo X client can consume — and no --validator or --miner flag is emitted. \
             Configure this node's validator key with the client's own tooling.",
        ),
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/roles/launchable/tests.rs"]
mod tests;
