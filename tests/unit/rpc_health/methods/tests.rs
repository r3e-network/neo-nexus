use serde_json::json;

use super::*;

/// A Neo X node asked `getversion` answers `-32601 method not found`, so a
/// healthy node probed with the wrong pair reports as an outage.
#[test]
fn each_family_is_asked_the_methods_its_clients_answer() {
    let n3 = probe_methods(ChainFamily::NeoN3);
    assert_eq!(n3.version, "getversion");
    assert_eq!(n3.height, "getblockcount");

    let neox = probe_methods(ChainFamily::NeoX);
    assert_eq!(neox.version, "web3_clientVersion");
    assert_eq!(neox.height, "eth_blockNumber");
}

/// No method name may be shared: if one leaked across, the bug would only show
/// up against a live node of the other family.
#[test]
fn the_two_families_share_no_method_name() {
    let n3 = probe_methods(ChainFamily::NeoN3);
    let neox = probe_methods(ChainFamily::NeoX);
    for method in [n3.version, n3.height] {
        assert_ne!(method, neox.version);
        assert_ne!(method, neox.height);
    }
}

#[test]
fn a_neo_n3_block_count_is_read_as_a_plain_number() {
    let n3 = probe_methods(ChainFamily::NeoN3);
    assert_eq!(n3.block_count(&json!(8_675_309)), Some(8_675_309));
    assert_eq!(n3.block_count(&json!("8675309")), Some(8_675_309));
    assert_eq!(n3.block_count(&json!(null)), None);
}

/// `eth_blockNumber` is the height of the latest block, one less than a count,
/// and hex-encoded. Reporting the raw value would make every Neo X node look
/// one block behind its Neo N3 neighbours in the same table.
#[test]
fn an_evm_block_number_is_decoded_and_turned_into_a_count() {
    let neox = probe_methods(ChainFamily::NeoX);
    assert_eq!(neox.block_count(&json!("0x0")), Some(1));
    assert_eq!(neox.block_count(&json!("0x10")), Some(17));
    assert_eq!(neox.block_count(&json!("0X10")), Some(17));
}

/// A decimal string is not a valid EVM QUANTITY. Guessing at one would turn
/// `"10"` into block 10 when the node meant 0x10 — a silently wrong height.
#[test]
fn a_malformed_evm_quantity_is_reported_as_unknown_rather_than_guessed() {
    let neox = probe_methods(ChainFamily::NeoX);
    for malformed in [json!("10"), json!("0xzz"), json!(17), json!(null)] {
        assert_eq!(neox.block_count(&malformed), None, "{malformed}");
    }
}

/// A chain longer than 2^64 blocks cannot exist, but the arithmetic must not
/// wrap into a smaller number if a node ever reports one.
#[test]
fn a_saturated_height_never_wraps_to_zero() {
    let neox = probe_methods(ChainFamily::NeoX);
    assert_eq!(
        neox.block_count(&json!("0xffffffffffffffff")),
        Some(u64::MAX)
    );
}
