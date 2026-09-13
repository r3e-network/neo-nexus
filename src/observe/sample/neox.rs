//! Reading a Neo X node.
//!
//! Neo X is an EVM sidechain, so every answer arrives as a `QUANTITY` — a
//! `0x`-prefixed, minimal-length hex string — rather than a number, and the
//! sync question is answered directly by `eth_syncing` rather than inferred.
//! That last point matters: on Neo X, "am I caught up" is a fact the node will
//! state, and the previous design instead tried to read it out of log text,
//! matching on strings neither client emits.

use serde_json::Value;

/// The methods, named once; see the note in the Neo N3 reader.
pub(super) const BLOCK_NUMBER: &str = "eth_blockNumber";
pub(super) const SYNCING: &str = "eth_syncing";
pub(super) const BLOCK_BY_NUMBER: &str = "eth_getBlockByNumber";
pub(super) const PEER_COUNT: &str = "net_peerCount";
pub(super) const CHAIN_ID: &str = "eth_chainId";
pub(super) const CLIENT_VERSION: &str = "web3_clientVersion";
pub(super) const TXPOOL_STATUS: &str = "txpool_status";

/// An EVM `QUANTITY`.
pub(super) fn hex_quantity(value: &Value) -> Option<u64> {
    let text = value.as_str()?;
    let digits = text
        .strip_prefix("0x")
        .or_else(|| text.strip_prefix("0X"))?;
    u64::from_str_radix(digits, 16).ok()
}

/// Blocks the node holds.
///
/// `eth_blockNumber` is the **height** of the newest block, so the count is one
/// more. Normalised to a count to match Neo N3's `getblockcount`.
pub(super) fn block_count(value: &Value) -> Option<u64> {
    hex_quantity(value).map(|height| height.saturating_add(1))
}

/// Whether the node says it is still catching up.
///
/// `eth_syncing` answers `false` when synced and an object of progress fields
/// when not. Anything else is unreadable rather than "synced" — assuming the
/// happy case from an unrecognised shape is how a syncing node comes to read as
/// healthy.
pub(super) fn syncing(value: &Value) -> Option<bool> {
    match value {
        Value::Bool(false) => Some(false),
        Value::Object(_) => Some(true),
        _ => None,
    }
}

/// The highest block the node is syncing toward, when it says it is syncing.
///
/// This is a reference head the node supplies about itself, available even on a
/// single-node workspace with nothing to compare against.
pub(super) fn sync_target(value: &Value) -> Option<u64> {
    value
        .as_object()?
        .get("highestBlock")
        .and_then(hex_quantity)
        .map(|height| height.saturating_add(1))
}

/// When the newest block was produced.
///
/// Unlike Neo N3, this is already **seconds**. The asymmetry is the trap; see
/// the note in the Neo N3 reader.
pub(super) fn block_time_unix(block: &Value) -> Option<u64> {
    block.get("timestamp").and_then(hex_quantity)
}

/// Connected peers. `net_peerCount` is a `QUANTITY`, not a number.
pub(super) fn peer_count(value: &Value) -> Option<u32> {
    hex_quantity(value).and_then(|count| u32::try_from(count).ok())
}

/// Transaction pool depth from `txpool_status`.
///
/// `pending` are executable now; `queued` are not yet. Mapped onto Neo N3's
/// verified/unverified split so one pair of columns serves both chains.
pub(super) fn pool_counts(value: &Value) -> (Option<u64>, Option<u64>) {
    (
        value.get("pending").and_then(hex_quantity),
        value.get("queued").and_then(hex_quantity),
    )
}

#[cfg(test)]
#[path = "../../../tests/unit/observe/neox_tests.rs"]
mod tests;
