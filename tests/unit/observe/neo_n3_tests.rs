use super::*;
use serde_json::json;

/// `getblockcount` is a count. The newest block's height is one less, and the
/// two are a block apart — which is exactly the size of ordinary propagation
/// jitter, so getting it backwards is invisible until it is not.
#[test]
fn block_count_reads_a_count_from_either_shape() {
    assert_eq!(block_count(&json!(6_245_100u64)), Some(6_245_100));
    // Some clients answer as a string.
    assert_eq!(block_count(&json!("6245100")), Some(6_245_100));
    assert_eq!(block_count(&json!("not a number")), None);
    assert_eq!(block_count(&json!(null)), None);
}

/// **Neo N3 reports block times in milliseconds.**
///
/// A missed division does not produce a slightly wrong figure. `1770000000000`
/// read as seconds is the year 58,092 — so chain lag comes out negative by
/// about fifty-four thousand years, every node reads `Stalled`, and the one
/// state this layer exists to detect becomes permanently and uselessly on.
#[test]
fn neo_n3_block_time_is_milliseconds_and_is_divided() {
    let header = json!({ "time": 1_770_000_000_123u64, "hash": "0xabc" });
    let seconds = block_time_unix(&header).expect("a header carries a time");
    assert_eq!(seconds, 1_770_000_000);

    // The guard that would catch a regression: the answer has to be a
    // plausible wall-clock instant, not a number four orders of magnitude out.
    assert!(
        (1_600_000_000..2_000_000_000).contains(&seconds),
        "block time {seconds} is not a plausible unix timestamp — the \
         milliseconds-to-seconds division has probably been lost"
    );
}

#[test]
fn a_header_without_a_time_yields_no_reading() {
    assert_eq!(block_time_unix(&json!({ "hash": "0xabc" })), None);
    assert_eq!(block_time_unix(&json!("not an object")), None);
}

/// `getversion` is the call whose answer was previously discarded. Its
/// `protocol` block is where every threshold in the product comes from.
#[test]
fn getversion_yields_the_protocol_constants_that_replace_hardcoded_thresholds() {
    let reply = json!({
        "useragent": "/Neo:3.9.2/",
        "protocol": {
            "network": 860_833_102u64,
            "msperblock": 15_000u64,
            "memorypoolmaxtransactions": 50_000u64,
            "validatorscount": 7u64,
        }
    });
    let protocol = protocol(&reply);
    assert_eq!(protocol.magic, Some(860_833_102));
    assert_eq!(protocol.ms_per_block, Some(15_000));
    assert_eq!(protocol.mempool_capacity, Some(50_000));
    assert_eq!(protocol.validators_count, Some(7));
    assert_eq!(protocol.user_agent.as_deref(), Some("/Neo:3.9.2/"));
}

/// The magic read here is the one the node *joined*, which is the whole point:
/// a node configured "private" that fell back to compiled-in MainNet defaults
/// reports MainNet's magic, and that mismatch is the only way to see it.
#[test]
fn the_magic_read_is_the_one_the_node_actually_joined() {
    let private = protocol(&json!({ "protocol": { "network": 1_230_000u64 } }));
    let mainnet = protocol(&json!({ "protocol": { "network": 860_833_102u64 } }));
    assert_ne!(private.magic, mainnet.magic);
}

/// A reply missing the protocol block yields nothing rather than defaults.
#[test]
fn a_version_reply_without_protocol_constants_yields_none_not_defaults() {
    let protocol = protocol(&json!({ "useragent": "/Neo:3.9.2/" }));
    assert_eq!(protocol.magic, None);
    assert_eq!(protocol.ms_per_block, None);
    assert_eq!(protocol.mempool_capacity, None);
    assert_eq!(protocol.validators_count, None);
}

#[test]
fn mempool_counts_read_both_lists_and_report_a_missing_one_as_unknown() {
    let reply = json!({
        "verified": ["0xa", "0xb", "0xc"],
        "unverified": ["0xd"],
    });
    let (verified, unverified) = mempool_counts(&reply);
    assert_eq!(verified.value(), Some(&3));
    assert_eq!(unverified.value(), Some(&1));

    let (verified, unverified) = mempool_counts(&json!({ "verified": [] }));
    assert_eq!(verified.value(), Some(&0));
    assert!(
        !unverified.is_known(),
        "an absent list must not be read as an empty one"
    );
}
