use super::*;

use crate::observe::evidence::{Evidence, Observation};

/// A sample at `at` with `height`, as the sampler would have produced it.
fn at(height: u64, at: u64) -> NodeSample {
    let mut sample = NodeSample::not_observable("node-1", at);
    sample.head_ok = true;
    sample.block_height = Observation::Known(
        height,
        Evidence::recorded("getblockcount", "result", height.to_string(), "", at),
    );
    sample
}

fn with_block_time(mut sample: NodeSample, block_time: u64) -> NodeSample {
    sample.head_block_time_unix = Observation::Known(
        block_time,
        Evidence::recorded("getblockheader", "time", block_time.to_string(), "", 0),
    );
    sample
}

const NOW: u64 = 1_770_000_000;

/// The stall witness that needs no reference node and no trust in the clock.
#[test]
fn time_since_the_height_last_moved_is_measured_from_the_change() {
    // Newest first. The height moved to 100 at T-120 and has not moved since.
    let history = [
        at(100, NOW - 15),
        at(100, NOW - 60),
        at(100, NOW - 120),
        at(99, NOW - 180),
    ];
    let derived = derive(&history, &ReferenceHead::SelfOnly, NOW);
    assert_eq!(
        derived.height_unchanged_seconds,
        Some(180),
        "the change is no earlier than the newest sample that predates it"
    );
}

/// A node stalled since before the workspace started still accumulates a
/// duration — otherwise the one case the layer exists for reports nothing.
#[test]
fn a_node_stalled_for_longer_than_the_window_still_reports_a_duration() {
    let history = [at(100, NOW - 15), at(100, NOW - 300), at(100, NOW - 600)];
    let derived = derive(&history, &ReferenceHead::SelfOnly, NOW);
    assert_eq!(derived.height_unchanged_seconds, Some(600));
}

/// With one sample there is nothing to compare, and the answer is the span so
/// far rather than zero — zero would read as "it just moved".
#[test]
fn a_single_sample_does_not_claim_the_height_just_changed() {
    let derived = derive(&[at(100, NOW - 42)], &ReferenceHead::SelfOnly, NOW);
    assert_eq!(derived.height_unchanged_seconds, Some(42));
}

/// A head with no reference is `None`, never `0`. Rendering the first as the
/// second tells an operator their single-node private chain is perfectly in
/// sync with a network that does not exist.
#[test]
fn head_lag_with_nothing_to_compare_against_is_unknown_not_zero() {
    let derived = derive(&[at(100, NOW)], &ReferenceHead::SelfOnly, NOW);
    assert_eq!(derived.head_lag, None);

    let derived = derive(
        &[at(100, NOW)],
        &ReferenceHead::Known {
            height: 112,
            source: "fleet median".to_string(),
        },
        NOW,
    );
    assert_eq!(derived.head_lag, Some(12));
}

/// Chain lag comes from the newest block's own timestamp, which the Neo N3
/// parser has already converted from milliseconds.
#[test]
fn chain_lag_is_the_age_of_the_newest_block() {
    let history = [with_block_time(at(100, NOW), NOW - 480)];
    let derived = derive(&history, &ReferenceHead::SelfOnly, NOW);
    assert_eq!(derived.chain_lag_seconds, Some(480));
}

/// A block dated in the future means a clock is wrong somewhere. Chain lag is
/// then dropped rather than reported as a negative number presented as
/// freshness.
#[test]
fn a_block_from_the_future_suspends_chain_lag_rather_than_going_negative() {
    let history = [with_block_time(at(100, NOW), NOW + 600)];
    let derived = derive(&history, &ReferenceHead::SelfOnly, NOW);
    assert!(derived.clock_suspect);
    assert_eq!(derived.chain_lag_seconds, None);

    // A few seconds of skew across a network is normal and not worth flagging.
    let history = [with_block_time(at(100, NOW), NOW + 5)];
    let derived = derive(&history, &ReferenceHead::SelfOnly, NOW);
    assert!(!derived.clock_suspect);
    assert_eq!(derived.chain_lag_seconds, Some(0));
}

/// A height going backwards is a real incident — a deep reorg, a restored
/// archive, a swapped data directory — and must never produce a negative rate.
#[test]
fn a_height_that_went_backwards_is_reported_and_suppresses_the_rate() {
    let history = [at(90, NOW), at(100, NOW - 60), at(99, NOW - 120)];
    let derived = derive(&history, &ReferenceHead::SelfOnly, NOW);
    assert_eq!(derived.height_regressed, Some((100, 90)));
    assert_eq!(
        derived.blocks_per_minute, None,
        "a regression must not be reported as a negative block rate"
    );
}

#[test]
fn block_rate_needs_a_real_window() {
    // Four blocks over two minutes is two per minute.
    let history = [at(104, NOW), at(102, NOW - 60), at(100, NOW - 120)];
    let derived = derive(&history, &ReferenceHead::SelfOnly, NOW);
    assert_eq!(derived.blocks_per_minute, Some(2.0));

    // Too short a window amplifies one late block into an alarming rate.
    let history = [at(104, NOW), at(100, NOW - 10)];
    assert_eq!(
        derive(&history, &ReferenceHead::SelfOnly, NOW).blocks_per_minute,
        None
    );
}

/// Headers ahead of blocks is a sync signal that needs nothing to compare
/// against — the node already knows about work it has not done.
#[test]
fn the_gap_between_headers_and_blocks_is_a_local_sync_signal() {
    let mut sample = at(100, NOW);
    sample.header_height = Observation::Known(
        4_320,
        Evidence::recorded("getblockheadercount", "result", "4320", "", NOW),
    );
    let derived = derive(&[sample], &ReferenceHead::SelfOnly, NOW);
    assert_eq!(derived.header_gap, Some(4_220));
}

/// Mempool pressure is measured against the capacity the node reported, which
/// replaces the compile-time 500/2000 thresholds that read "Elevated" on a
/// chain configured for 5,000-transaction blocks.
#[test]
fn mempool_pressure_is_relative_to_the_capacity_the_node_reported() {
    let evidence = |value: u64| Evidence::recorded("getversion", "f", value.to_string(), "", 0);
    let mut sample = at(100, NOW);
    sample.mempool_capacity = Observation::Known(50_000, evidence(50_000));
    sample.mempool_verified = Observation::Known(500, evidence(500));
    sample.mempool_unverified = Observation::Known(0, evidence(0));

    let derived = derive(&[sample.clone()], &ReferenceHead::SelfOnly, NOW);
    let utilisation = derived.mempool_utilisation.expect("capacity was reported");
    assert!(
        utilisation < 0.02,
        "500 of 50000 is 1%, not congestion: {utilisation}"
    );

    // Without a reported capacity there is no ratio, and none is invented.
    sample.mempool_capacity = Observation::Unknown(crate::observe::NotSampled::NeverSampled);
    assert_eq!(
        derive(&[sample], &ReferenceHead::SelfOnly, NOW).mempool_utilisation,
        None
    );
}

/// Every threshold downstream scales from the node's own reported interval
/// rather than from a constant in the binary.
#[test]
fn the_block_interval_comes_from_the_node() {
    let mut sample = at(100, NOW);
    sample.ms_per_block = Observation::Known(
        15_000,
        Evidence::recorded("getversion", "protocol.msperblock", "15000", "", NOW),
    );
    assert_eq!(
        derive(&[sample], &ReferenceHead::SelfOnly, NOW).block_interval_seconds,
        Some(15)
    );
}

#[test]
fn no_samples_derives_nothing_rather_than_zeroes() {
    let derived = derive(&[], &ReferenceHead::SelfOnly, NOW);
    assert_eq!(derived, Derived::default());
    assert_eq!(derived.height_unchanged_seconds, None);
    assert_eq!(derived.chain_lag_seconds, None);
    assert_eq!(derived.blocks_per_minute, None);
}
