//! Reading a Neo N3 node.

use serde_json::Value;

use crate::observe::evidence::{NotSampled, Observation};

/// The methods, named once.
///
/// `methods` below and the collector both read these, so the table a test
/// asserts against and the calls actually issued cannot drift apart — which is
/// the failure mode this whole module exists to correct, applied to itself.
pub(super) const BLOCK_COUNT: &str = "getblockcount";
pub(super) const HEADER_COUNT: &str = "getblockheadercount";
pub(super) const BLOCK_HEADER: &str = "getblockheader";
pub(super) const CONNECTION_COUNT: &str = "getconnectioncount";
pub(super) const VERSION: &str = "getversion";
pub(super) const RAW_MEMPOOL: &str = "getrawmempool";

/// Blocks the node holds.
///
/// `getblockcount` is a **count**, so the newest block's height is one less.
/// Neo X's `eth_blockNumber` is already a height. Both are normalised to a
/// count here so that one operator-facing number means the same thing on both
/// chains, and the comment exists because getting this backwards produces an
/// off-by-one that looks exactly like propagation jitter.
pub(super) fn block_count(value: &Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_str().and_then(|text| text.parse().ok()))
}

/// When the newest block was produced.
///
/// **Neo N3 reports block times in milliseconds.** Neo X reports seconds. A
/// missed division here does not produce a slightly wrong figure — it makes
/// every Neo N3 node read as about fifty-four thousand years behind the chain,
/// and every one of them `Stalled`. This is the single most consequential unit
/// conversion in the layer and it has its own test.
pub(super) fn block_time_unix(header: &Value) -> Option<u64> {
    let milliseconds = header.get("time")?.as_u64()?;
    Some(milliseconds / 1000)
}

/// Protocol constants, from the call whose answer used to be discarded.
pub(super) struct Protocol {
    pub(super) magic: Option<u64>,
    pub(super) ms_per_block: Option<u64>,
    pub(super) mempool_capacity: Option<u64>,
    pub(super) validators_count: Option<u32>,
    pub(super) user_agent: Option<String>,
}

/// Read `getversion`.
///
/// The `protocol` block is the point of the call. Previously only `useragent`
/// was kept, which is why every threshold in the product had to be a
/// compile-time constant: the node was being told what its block interval was
/// instead of being asked.
pub(super) fn protocol(value: &Value) -> Protocol {
    let protocol = value.get("protocol");
    Protocol {
        magic: protocol
            .and_then(|p| p.get("network"))
            .and_then(Value::as_u64),
        ms_per_block: protocol
            .and_then(|p| p.get("msperblock"))
            .and_then(Value::as_u64),
        mempool_capacity: protocol
            .and_then(|p| p.get("memorypoolmaxtransactions"))
            .and_then(Value::as_u64),
        validators_count: protocol
            .and_then(|p| p.get("validatorscount"))
            .and_then(Value::as_u64)
            .and_then(|count| u32::try_from(count).ok()),
        user_agent: value
            .get("useragent")
            .and_then(Value::as_str)
            .map(str::to_string),
    }
}

/// Mempool depth, verified and unverified.
///
/// `getrawmempool(true)` answers with both lists. The counts are what matter;
/// the transaction hashes are discarded rather than stored, because a node
/// manager has no use for them and they are unbounded.
pub(super) fn mempool_counts(value: &Value) -> (Observation<u64>, Observation<u64>) {
    let read = |field: &'static str| -> Observation<u64> {
        match value.get(field).and_then(Value::as_array) {
            Some(entries) => Observation::Known(
                entries.len() as u64,
                crate::observe::evidence::Evidence::recorded(
                    RAW_MEMPOOL,
                    field,
                    entries.len().to_string(),
                    String::new(),
                    0,
                ),
            ),
            None => Observation::Unknown(NotSampled::CallFailed {
                method: RAW_MEMPOOL,
                detail: format!("reply carried no {field} array"),
            }),
        }
    };
    (read("verified"), read("unverified"))
}

#[cfg(test)]
#[path = "../../../tests/unit/observe/neo_n3_tests.rs"]
mod tests;
