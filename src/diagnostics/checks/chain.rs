//! What chain a node believes it is on.
//!
//! Neo N3 proves this with a network magic that every peer handshake checks: a
//! node with the wrong one is refused by every peer and the mistake is loud.
//!
//! Neo X does guard it, but later and more quietly. Its identity is an EIP-155
//! chain id plus a genesis block, and a peer whose genesis differs is dropped
//! during the handshake — so a node initialised from the wrong genesis does not
//! sync a different chain, it syncs *nothing*: it sits at block 0 with no peers,
//! which reads as a network problem rather than a wrong-genesis problem. So the
//! expected block-0 hash is published here, where an operator can compare it
//! against `eth_getBlockByNumber(0)` instead of hunting for a firewall.
//!
//! The sharper hazard is the other direction. Neither client refuses to start
//! without being told which chain it is on: neox-rs defaults `--chain` to Neo X
//! MainNet, and Neo X Geth's `NetworkId` alone does not pin a genesis. A node an
//! operator labelled **Private** therefore joins a real public chain unless they
//! supply their own spec, so that is a launch blocker rather than a note.

use crate::{
    config::{neox_block_period_secs, neox_chain_id, neox_genesis_hash, neox_validator_count},
    diagnostics::{CheckSeverity, DiagnosticCheck, DiagnosticResolution},
    types::{ChainFamily, Network, NodeConfig, NodeType},
};

pub(in crate::diagnostics) fn chain_identity_checks(node: &NodeConfig) -> Vec<DiagnosticCheck> {
    if node.node_type.family() != ChainFamily::NeoX {
        return Vec::new();
    }

    let mut checks = vec![
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
    ];
    checks.extend(private_chain_spec_check(node));
    checks
}

/// A private Neo X network must be given its own chain spec, explicitly.
///
/// NeoNexus generates no Neo X genesis — an allocation it invented would be a
/// chain of one — and the clients do not fail closed: omitting the chain gets
/// Neo X MainNet, not an error. So a node marked Private with no operator-
/// supplied chain in its launch arguments is stopped here, because the failure
/// it would otherwise reach is silent and involves real MainNet keys.
fn private_chain_spec_check(node: &NodeConfig) -> Option<DiagnosticCheck> {
    if node.network != Network::Private {
        return None;
    }
    if has_chain_argument(&node.args) {
        return Some(DiagnosticCheck::new(
            CheckSeverity::Pass,
            "Private chain spec",
            "The launch arguments name the chain spec this private network runs.".to_string(),
            DiagnosticResolution::NodeStudio,
        ));
    }
    Some(DiagnosticCheck::new(
        CheckSeverity::Critical,
        "Private chain spec",
        format!(
            "A private Neo X node must be launched with its own chain spec. Add `{}` to this \
             node's arguments; without it {} joins Neo X MainNet, and keys used on it sign \
             MainNet-valid transactions.",
            chain_flag_hint(node.node_type),
            node.node_type,
        ),
        DiagnosticResolution::NodeStudio,
    ))
}

/// Whether the operator already named a chain. Matches both `--flag value` and
/// `--flag=value`, and Geth's `--networkid` as well as Reth's `--chain`.
fn has_chain_argument(args: &[String]) -> bool {
    args.iter().any(|arg| {
        ["--chain", "--networkid", "--datadir.chain"]
            .iter()
            .any(|flag| arg == flag || arg.starts_with(&format!("{flag}=")))
    })
}

fn chain_flag_hint(node_type: NodeType) -> &'static str {
    match node_type {
        // Geth takes the chain from a datadir initialised by `geth init`, and
        // pins the id separately; naming the id is the part it can express.
        NodeType::NeoXGeth => "--networkid <your chain id>",
        _ => "--chain <your-chainspec>.json",
    }
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
