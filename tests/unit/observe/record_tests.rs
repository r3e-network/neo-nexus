use super::*;

use crate::observe::{Cause, HealthState, NextStep, Verdict};

fn verdict() -> Verdict {
    Verdict::new(
        HealthState::Stalled,
        "height 8421 has not advanced in 600s, and the node is still answering",
        Vec::new(),
        NextStep::here("Read the log", "/logs?node=node-1"),
    )
    .with_cause(Some(Cause::NoPeers))
}

/// The cause is recorded as the sentence an operator reads, not as a code they
/// would then have to look up. Telling someone to "check peers" when the peer
/// count is one column away in the same row is a design failure.
#[test]
fn a_recorded_verdict_keeps_the_sentence_that_explains_it() {
    let recorded = NodeHealth::from_verdict("node-1", &verdict(), 1_770_000_000, 1_770_000_600);
    assert_eq!(recorded.state, HealthState::Stalled);
    assert_eq!(recorded.reason, verdict().reason());
    assert_eq!(
        recorded.cause.as_deref(),
        Some("The node has no connected peers.")
    );
    assert_eq!(
        recorded.next,
        NextStep::here("Read the log", "/logs?node=node-1")
    );
}

/// How long a state has held and how fresh the judgement is are two different
/// numbers, and only one of them is about the node. A surface that fuses them
/// reports "stalled, checked 15 seconds ago" — which reads like reassurance
/// about a node that has been stuck for ten minutes.
#[test]
fn how_long_it_has_been_wrong_is_not_how_recently_it_was_checked() {
    let recorded = NodeHealth::from_verdict("node-1", &verdict(), 1_770_000_000, 1_770_000_600);
    assert_eq!(recorded.held_for_seconds(1_770_000_615), 615);
    assert_eq!(recorded.evaluated_seconds_ago(1_770_000_615), 15);
}

/// A clock that went backwards must not produce a negative duration rendered as
/// an enormous one.
#[test]
fn a_timestamp_in_the_future_reads_as_no_elapsed_time() {
    let recorded = NodeHealth::from_verdict("node-1", &verdict(), 1_770_000_600, 1_770_000_600);
    assert_eq!(recorded.held_for_seconds(1_770_000_000), 0);
    assert_eq!(recorded.evaluated_seconds_ago(1_770_000_000), 0);
}

/// The first verdict on a node came from nowhere, and its summary has to say so
/// rather than inventing a previous state it was never in.
#[test]
fn the_first_verdict_is_not_rendered_as_a_transition() {
    let first = HealthTransition {
        node_id: "node-1".to_string(),
        at_unix: 1_770_000_000,
        from: None,
        to: HealthState::Healthy,
        reason: "at height 8421, answering, and keeping up".to_string(),
    };
    assert!(!first.summary().contains('→'));

    let later = HealthTransition {
        from: Some(HealthState::Healthy),
        to: HealthState::Stalled,
        ..first
    };
    assert!(later.summary().starts_with("Healthy → Stalled:"));
}
