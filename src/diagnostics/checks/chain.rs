//! What chain a node believes it is on.
//!
//! Neo N3 proves this with a network magic that every peer handshake checks: a
//! node with the wrong one is refused by every peer and the mistake is loud.
//!
//! Neo X has no such handshake guard. Its identity is an EIP-155 chain id plus
//! a genesis block, and neither client verifies that the two agree with the
//! network it was pointed at. A Neo X Geth node whose data directory was
//! initialised from the wrong genesis syncs a *different chain* while still
//! reporting the configured chain id — it looks healthy, it produces blocks,
//! and nothing it signs is valid anywhere. So the anchor is published here,
//! where an operator can compare it against `eth_getBlockByNumber(0)`.

use crate::{
    config::{neox_block_period_secs, neox_chain_id, neox_genesis_hash, neox_validator_count},
    diagnostics::{CheckSeverity, DiagnosticCheck, DiagnosticResolution},
    types::{ChainFamily, NodeConfig},
};

pub(in crate::diagnostics) fn chain_identity_checks(node: &NodeConfig) -> Vec<DiagnosticCheck> {
    if node.node_type.family() != ChainFamily::NeoX {
        return Vec::new();
    }

    vec![
        DiagnosticCheck::new(
            CheckSeverity::Pass,
            "Chain identity",
            format!(
                "Neo X {}: chain id {}, {}s blocks, {} dBFT consensus nodes.",
                node.network,
                neox_chain_id(node.network, None),
                neox_block_period_secs(node.network),
                neox_validator_count(node.network),
            ),
            DiagnosticResolution::ConfigWorkspace,
        ),
        genesis_check(node),
    ]
}

/// Neither client checks its genesis against the chain id it was given, so this
/// is the only place the mismatch can be caught before the node has synced a
/// chain nobody else is on.
fn genesis_check(node: &NodeConfig) -> DiagnosticCheck {
    let detail = match neox_genesis_hash(node.network) {
        Some(hash) => format!(
            "Verify block 0 hashes to {hash}. A data directory initialised from another genesis \
             syncs a different chain while still reporting the configured chain id."
        ),
        None => "A private Neo X network needs its own genesis file, shared byte for byte by \
                 every member; NeoNexus does not generate one, because an invented allocation \
                 produces a chain of one."
            .to_string(),
    };
    DiagnosticCheck::new(
        CheckSeverity::Warning,
        "Genesis anchor",
        detail,
        DiagnosticResolution::ConfigWorkspace,
    )
}

#[cfg(test)]
#[path = "../../../tests/unit/diagnostics/checks/chain/tests.rs"]
mod tests;
