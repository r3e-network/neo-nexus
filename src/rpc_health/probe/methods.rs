//! Which JSON-RPC methods prove a node is alive, per chain family.
//!
//! The two families share the JSON-RPC envelope and nothing else. Asking a
//! Neo X node for `getversion` gets a `-32601 method not found`, so a healthy
//! Neo X node probed with the Neo N3 pair reports **Unreachable** — the worst
//! kind of wrong answer, because it looks like an outage rather than a bug.
//!
//! Both Neo X methods are in namespaces that are on by default: Reth's
//! `STANDARD_MODULES` is `[eth, net, web3]`, and the generated Neo X Geth
//! config lists `eth`, `net`, `web3` and `txpool` explicitly.

use serde_json::Value;

use crate::types::ChainFamily;

/// The method pair a probe calls, and how to read their answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ProbeMethods {
    /// Reports what software the node is running.
    pub(super) version: &'static str,
    /// Reports how far the node has synced.
    pub(super) height: &'static str,
    /// The "am I still catching up" call, where the family has one. A height
    /// alone cannot tell "at head" from "stalled a few blocks behind".
    pub(super) syncing: Option<&'static str>,
    family: ChainFamily,
}

pub(super) fn probe_methods(family: ChainFamily) -> ProbeMethods {
    match family {
        ChainFamily::NeoN3 => ProbeMethods {
            version: "getversion",
            height: "getblockcount",
            syncing: None,
            family,
        },
        ChainFamily::NeoX => ProbeMethods {
            version: "web3_clientVersion",
            height: "eth_blockNumber",
            syncing: Some("eth_syncing"),
            family,
        },
    }
}

impl ProbeMethods {
    /// The number of blocks the node holds, from whichever answer it gave.
    ///
    /// The two families do not report the same quantity: `getblockcount` is a
    /// count, `eth_blockNumber` is the height of the latest block — one less,
    /// and hex-encoded. Normalising here is what lets one operator-facing
    /// number mean the same thing on both chains.
    pub(super) fn block_count(self, value: &Value) -> Option<u64> {
        match self.family {
            ChainFamily::NeoN3 => value
                .as_u64()
                .or_else(|| value.as_str().and_then(|text| text.parse().ok())),
            ChainFamily::NeoX => hex_quantity(value).map(|height| height.saturating_add(1)),
        }
    }
}

/// Reads a sync verdict from the family's syncing call, if it has one.
///
/// `eth_syncing` answers `false` at head, or an object of block numbers while
/// still catching up. Anything else — `null`, a string — is not a verdict and
/// leaves the probe's judgement to the two scored methods.
pub(super) fn syncing_verdict(value: &Value) -> Option<bool> {
    if value.as_bool() == Some(false) {
        return Some(false);
    }
    if value.is_object() {
        return Some(true);
    }
    None
}

/// An EVM `QUANTITY`: `0x`-prefixed, minimal-length hex.
fn hex_quantity(value: &Value) -> Option<u64> {
    let text = value.as_str()?;
    let digits = text
        .strip_prefix("0x")
        .or_else(|| text.strip_prefix("0X"))?;
    u64::from_str_radix(digits, 16).ok()
}

#[cfg(test)]
#[path = "../../../tests/unit/rpc_health/methods/tests.rs"]
mod tests;
