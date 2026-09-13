use super::*;
use serde_json::json;

#[test]
fn quantities_are_hex_and_anything_else_is_unreadable() {
    assert_eq!(hex_quantity(&json!("0x1c2b3d")), Some(1_846_077));
    assert_eq!(hex_quantity(&json!("0x0")), Some(0));
    assert_eq!(hex_quantity(&json!("0X1F")), Some(31));
    // A decimal number is not a QUANTITY; reading it as one would be wrong by
    // orders of magnitude rather than slightly.
    assert_eq!(hex_quantity(&json!(1_846_077u64)), None);
    assert_eq!(hex_quantity(&json!("1846077")), None);
    assert_eq!(hex_quantity(&json!(null)), None);
}

/// `eth_blockNumber` is a height; Neo N3's `getblockcount` is a count. Both are
/// normalised to a count so a single column means one thing on both chains.
#[test]
fn block_count_normalises_a_height_to_a_count() {
    assert_eq!(block_count(&json!("0x0")), Some(1));
    assert_eq!(block_count(&json!("0x1c2b3d")), Some(1_846_078));
}

/// `eth_syncing` answers the sync question directly. The previous design tried
/// to infer it from log text, matching on strings neither Neo X client emits.
#[test]
fn syncing_is_read_from_the_node_rather_than_inferred() {
    assert_eq!(syncing(&json!(false)), Some(false));
    assert_eq!(
        syncing(&json!({ "currentBlock": "0x10", "highestBlock": "0x20" })),
        Some(true)
    );
}

/// An unrecognised shape is unreadable, not "synced". Assuming the happy case
/// from a reply we do not understand is how a syncing node comes to read green.
#[test]
fn an_unrecognised_sync_reply_is_unreadable_rather_than_assumed_healthy() {
    assert_eq!(syncing(&json!(true)), None);
    assert_eq!(syncing(&json!("false")), None);
    assert_eq!(syncing(&json!(null)), None);
}

/// When a node says it is syncing it also says what it is syncing toward — a
/// reference head it supplies about itself, available with nothing to compare
/// against.
#[test]
fn a_syncing_node_reports_the_head_it_is_chasing() {
    let progress = json!({ "currentBlock": "0x10", "highestBlock": "0x20" });
    assert_eq!(sync_target(&progress), Some(0x21));
    assert_eq!(sync_target(&json!(false)), None);
    assert_eq!(sync_target(&json!({ "currentBlock": "0x10" })), None);
}

/// Neo X block timestamps are **seconds** where Neo N3's are milliseconds.
/// The asymmetry is the trap; this pins the Neo X side of it.
#[test]
fn neox_block_time_is_already_seconds() {
    let block = json!({ "timestamp": "0x6987cd00", "number": "0x1c2b3d" });
    let seconds = block_time_unix(&block).expect("a block carries a timestamp");
    assert_eq!(seconds, 0x6987_cd00);
    assert!(
        (1_600_000_000..2_000_000_000).contains(&seconds),
        "block time {seconds} is not a plausible unix timestamp — a division \
         has probably been applied that belongs only to Neo N3"
    );
}

#[test]
fn peer_count_is_a_quantity() {
    assert_eq!(peer_count(&json!("0xe")), Some(14));
    assert_eq!(peer_count(&json!("0x0")), Some(0));
    assert_eq!(peer_count(&json!(14u64)), None);
}

/// `txpool_status` splits executable from not-yet-executable, which maps onto
/// Neo N3's verified/unverified so one pair of columns serves both chains.
#[test]
fn pool_counts_map_pending_and_queued_onto_the_shared_columns() {
    let status = json!({ "pending": "0x55", "queued": "0xc" });
    assert_eq!(pool_counts(&status), (Some(0x55), Some(0xc)));
    assert_eq!(
        pool_counts(&json!({ "pending": "0x55" })),
        (Some(0x55), None)
    );
}
