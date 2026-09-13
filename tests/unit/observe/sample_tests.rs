use super::*;

use crate::observe::evidence::NotSampled;

/// Each class is asked at its own rate. One period for everything means either
/// paying for `getversion` four times a minute or learning the block interval
/// once an hour — and the block interval is what every stall threshold is
/// derived from.
#[test]
fn classes_are_sampled_at_rates_that_match_what_they_measure() {
    assert_eq!(SampleClass::Head.default_period().as_secs(), 15);
    assert_eq!(SampleClass::Peers.default_period().as_secs(), 15);
    assert_eq!(SampleClass::HeadTime.default_period().as_secs(), 60);
    assert_eq!(SampleClass::Pool.default_period().as_secs(), 120);
    assert_eq!(SampleClass::Identity.default_period().as_secs(), 900);

    // Protocol constants change only when a node restarts onto a different
    // config, so asking every round would be pure cost.
    assert!(SampleClass::Identity.default_period() > SampleClass::Head.default_period() * 30);
}

/// Only the head class decides whether a node is answering. Every other class
/// failing is a gap in what is known about a node that is otherwise responding.
///
/// This is the correction to a probe that derived status from "how many of two
/// calls answered", under which one unimplemented method condemned the node.
#[test]
fn only_the_head_class_speaks_for_liveness() {
    assert!(SampleClass::Head.is_liveness());
    for quiet in [
        SampleClass::HeadTime,
        SampleClass::Peers,
        SampleClass::Identity,
        SampleClass::Pool,
    ] {
        assert!(
            !quiet.is_liveness(),
            "{quiet:?} failing must not make a responding node read as down"
        );
    }
}

#[test]
fn every_class_has_a_distinct_stable_key() {
    let mut keys = std::collections::BTreeSet::new();
    for class in SampleClass::ALL {
        assert!(keys.insert(class.key()), "{class:?} has a duplicate key");
    }
}

/// The two families share the JSON-RPC envelope and nothing else. Asking a
/// Neo X node for `getversion` earns a `-32601`, which the previous probe
/// counted as a failure — so a healthy Neo X node read as unreachable.
///
/// Each method is named once, as a constant both the reader and the collector
/// use, so the table this asserts against and the calls actually issued cannot
/// drift apart.
#[test]
fn the_two_families_are_asked_in_their_own_vocabularies() {
    use crate::observe::sample::{neo_n3, neox};

    let n3 = [
        neo_n3::BLOCK_COUNT,
        neo_n3::HEADER_COUNT,
        neo_n3::BLOCK_HEADER,
        neo_n3::CONNECTION_COUNT,
        neo_n3::VERSION,
        neo_n3::RAW_MEMPOOL,
    ];
    let evm = [
        neox::BLOCK_NUMBER,
        neox::SYNCING,
        neox::BLOCK_BY_NUMBER,
        neox::PEER_COUNT,
        neox::CHAIN_ID,
        neox::CLIENT_VERSION,
        neox::TXPOOL_STATUS,
    ];
    for method in n3 {
        assert!(
            !evm.contains(&method),
            "{method} is asked of both families; they share no methods"
        );
        assert!(!method.contains('_'), "{method} is not Neo N3 spelling");
    }
    for method in evm {
        assert!(
            method.contains('_'),
            "{method} is not an EVM namespaced method"
        );
    }
}

/// `txpool_status` is the cheap way to read a Neo X pool. The alternative —
/// `eth_getBlockTransactionCountByNumber(["pending"])` — forces geth to
/// *construct* a pending block, and the previous implementation preferred it.
#[test]
fn the_neox_pool_is_read_by_the_cheap_method() {
    use crate::observe::sample::neox;
    assert_eq!(neox::TXPOOL_STATUS, "txpool_status");
}

/// A node with no RPC port is not a node that failed. Both render as absent,
/// but for reasons that lead an operator to different actions.
#[test]
fn a_node_that_cannot_be_asked_reads_differently_from_one_that_did_not_answer() {
    let disabled = NodeSample::not_observable("node-1", 1_770_000_000);
    let down = NodeSample::unreachable(
        "node-2",
        1_770_000_000,
        "http://127.0.0.1:10332",
        NotSampled::CallFailed {
            method: "getblockcount",
            detail: "connection refused".to_string(),
        },
    );

    assert!(!disabled.head_ok && !down.head_ok);
    let disabled_text = disabled.block_height.render(|h| h.to_string());
    let down_text = down.block_height.render(|h| h.to_string());
    assert_ne!(disabled_text, down_text);
    assert!(disabled_text.contains("RPC is disabled"));
    assert!(down_text.contains("connection refused"));

    // And neither renders as a height.
    for text in [disabled_text, down_text] {
        assert!(text.parse::<u64>().is_err());
    }
}

/// Grouping is by the chain a node *joined*, not the one it was configured
/// with — which is what lets head lag be computed before a networks table
/// exists, and stops a node on a private magic being compared against MainNet.
#[test]
fn nodes_are_grouped_by_the_chain_they_actually_joined() {
    use crate::observe::evidence::{Evidence, Observation};
    use crate::types::ChainFamily;

    let mut sample = NodeSample::not_observable("node-1", 1_770_000_000);
    assert_eq!(
        sample.chain_key(ChainFamily::NeoN3),
        None,
        "a node that has not reported its magic cannot be grouped yet"
    );

    sample.observed_magic = Observation::Known(
        1_230_000,
        Evidence::recorded("getversion", "protocol.network", "1230000", "", 0),
    );
    let private = sample.chain_key(ChainFamily::NeoN3);

    sample.observed_magic = Observation::Known(
        860_833_102,
        Evidence::recorded("getversion", "protocol.network", "860833102", "", 0),
    );
    let mainnet = sample.chain_key(ChainFamily::NeoN3);

    assert_ne!(private, mainnet);
}
