//! The guard chain, as state vectors.
//!
//! Deliberately not fixtures copied out of a parser. The sync detection this
//! replaces was covered by a test whose log lines were written to match the
//! parser reading them, so both agreed and neither matched what the clients
//! actually emit. These inputs are constructed from the states themselves.

use super::*;

use crate::observe::{
    derive::Derived,
    evidence::{Evidence, NotSampled, Observation},
    sample::NodeSample,
};

const NOW: u64 = 1_770_000_000;

fn answering(height: u64, at: u64) -> NodeSample {
    let mut sample = NodeSample::not_observable("node-1", at);
    sample.head_ok = true;
    sample.head_latency_ms = Some(12);
    sample.block_height = Observation::Known(
        height,
        Evidence::recorded("getblockcount", "result", height.to_string(), "", at),
    );
    sample.peers_connected = Observation::Known(
        8,
        Evidence::recorded("getconnectioncount", "result", "8", "", at),
    );
    sample
}

fn inputs<'a>(latest: Option<&'a NodeSample>, derived: &'a Derived) -> HealthInputs<'a> {
    HealthInputs {
        now_unix: NOW,
        process_running: true,
        wants_running: true,
        uptime_seconds: Some(3_600),
        starting_grace_seconds: 60,
        latest,
        derived,
        consecutive_failures: 0,
        expected_peers: Some(3),
        policy: HealthPolicy::for_block_interval(Some(15)),
    }
}

/// **The failure this whole layer exists to catch.**
///
/// The node answers RPC promptly. Its process is up. A process watchdog sees
/// nothing wrong. Its height has not moved in seven minutes.
#[test]
fn a_node_that_answers_promptly_and_is_going_nowhere_is_stalled() {
    let latest = answering(6_245_100, NOW);
    let derived = Derived {
        height_unchanged_seconds: Some(420),
        chain_lag_seconds: Some(430),
        ..Derived::default()
    };
    let verdict = classify(&inputs(Some(&latest), &derived));

    assert_eq!(verdict.state(), HealthState::Stalled);
    assert_eq!(verdict.scope(), Some(StallScope::Node));
    assert!(
        verdict.reason().contains("420s"),
        "the operator needs the number: {}",
        verdict.reason()
    );
    assert!(
        verdict.reason().contains("6245100"),
        "and the height it is stuck at: {}",
        verdict.reason()
    );
    assert!(
        !verdict.evidence().is_empty(),
        "a verdict must show its working"
    );
}

/// A single slow block is not an incident. The threshold scales from the
/// node's own interval, so a four-second chain and a fifteen-second chain are
/// not judged by the same number.
#[test]
fn one_slow_block_is_not_a_stall_and_the_threshold_follows_the_chain() {
    let latest = answering(6_245_100, NOW);
    let derived = Derived {
        height_unchanged_seconds: Some(45),
        ..Derived::default()
    };
    assert_eq!(
        classify(&inputs(Some(&latest), &derived)).state(),
        HealthState::Healthy
    );

    // Twenty blocks, clamped. Neo N3's fifteen-second blocks give five
    // minutes; Neo X's four-second blocks give eighty seconds. The floor only
    // bites on chains faster than three-second blocks, so that a very fast
    // chain does not page on a single missed round.
    assert_eq!(
        HealthPolicy::for_block_interval(Some(15)).stall_seconds,
        300
    );
    assert_eq!(HealthPolicy::for_block_interval(Some(4)).stall_seconds, 80);
    assert_eq!(HealthPolicy::for_block_interval(Some(1)).stall_seconds, 60);
    // And a chain with very long blocks is capped, so a stall is still caught
    // within fifteen minutes however slow the chain is configured to be.
    assert_eq!(
        HealthPolicy::for_block_interval(Some(600)).stall_seconds,
        900
    );
    // With no reported interval the Neo N3 default is assumed, and said so.
    assert_eq!(HealthPolicy::for_block_interval(None).stall_seconds, 300);
}

/// Zero peers outranks the stall it causes, because the operator handed the
/// symptom would go looking for the cause anyway.
#[test]
fn no_peers_outranks_the_stall_it_causes_and_names_itself_as_the_cause() {
    let mut latest = answering(6_245_100, NOW);
    latest.peers_connected = Observation::Known(
        0,
        Evidence::recorded("getconnectioncount", "result", "0", "", NOW),
    );
    let derived = Derived {
        height_unchanged_seconds: Some(600),
        ..Derived::default()
    };
    let verdict = classify(&inputs(Some(&latest), &derived));
    assert_eq!(verdict.state(), HealthState::Isolated);
    assert_eq!(verdict.suspected_cause(), Some(&Cause::NoPeers));
}

/// A stall reports the cause from the same round's evidence rather than
/// telling the operator to go and look for one.
#[test]
fn a_stall_attributes_its_cause_from_measurements_already_in_hand() {
    let mut latest = answering(6_245_100, NOW);
    latest.peers_connected = Observation::Known(
        1,
        Evidence::recorded("getconnectioncount", "result", "1", "", NOW),
    );
    let derived = Derived {
        height_unchanged_seconds: Some(600),
        ..Derived::default()
    };
    let verdict = classify(&inputs(Some(&latest), &derived));
    assert_eq!(verdict.state(), HealthState::Stalled);
    assert_eq!(
        verdict.suspected_cause(),
        Some(&Cause::PeersBelowExpected {
            connected: 1,
            expected: 3
        })
    );
}

/// A node that is behind and *moving* is syncing; one that is behind and not
/// moving is stalled. The ordering is what keeps the second from hiding inside
/// the first.
#[test]
fn behind_and_moving_is_syncing_behind_and_stuck_is_stalled() {
    let latest = answering(1_000, NOW);
    let catching_up = Derived {
        head_lag: Some(4_200),
        height_unchanged_seconds: Some(10),
        blocks_per_minute: Some(240.0),
        ..Derived::default()
    };
    assert_eq!(
        classify(&inputs(Some(&latest), &catching_up)).state(),
        HealthState::Syncing
    );

    let stuck = Derived {
        head_lag: Some(4_200),
        height_unchanged_seconds: Some(600),
        blocks_per_minute: Some(0.0),
        ..Derived::default()
    };
    assert_eq!(
        classify(&inputs(Some(&latest), &stuck)).state(),
        HealthState::Stalled
    );
}

/// Initial sync, which every node does exactly once and which must not page
/// anyone.
///
/// A node restoring from a snapshot is hours behind the chain head, so its
/// newest block is hours old and the chain-lag witness fires — and `Stalled`
/// outranks `Syncing`. The height climbing every round is what separates the
/// two, and it has to be consulted before the lag is believed.
#[test]
fn a_node_working_through_its_initial_sync_is_syncing_not_stalled() {
    let latest = answering(1_000, NOW);
    let restoring = Derived {
        // Six hours behind the head, and climbing.
        chain_lag_seconds: Some(21_600),
        head_lag: Some(1_440),
        height_unchanged_seconds: Some(15),
        height_advanced_in_window: true,
        ..Derived::default()
    };
    assert_eq!(
        classify(&inputs(Some(&latest), &restoring)).state(),
        HealthState::Syncing
    );

    // The same distance from the head, with the height no longer moving, is the
    // stall the witness is for.
    let stuck = Derived {
        height_advanced_in_window: false,
        ..restoring
    };
    assert_eq!(
        classify(&inputs(Some(&latest), &stuck)).state(),
        HealthState::Stalled
    );
}

/// Neo X answers the sync question directly, so nothing has to be inferred.
#[test]
fn a_neox_node_that_says_it_is_syncing_is_believed() {
    let mut latest = answering(1_000, NOW);
    latest.syncing = Observation::Known(
        true,
        Evidence::recorded("eth_syncing", "result", "{…}", "", NOW),
    );
    let derived = Derived {
        height_unchanged_seconds: Some(5),
        ..Derived::default()
    };
    assert_eq!(
        classify(&inputs(Some(&latest), &derived)).state(),
        HealthState::Syncing
    );
}

/// A crashed node reads as unreachable, not as unchecked.
///
/// Nothing polls a node the workspace already believes is down, so its
/// consecutive-failure count never rises and the `Unreachable` threshold is
/// never met. Without a guard on the process itself, a node that died reads
/// "Not checked" — which is exactly the shape of an incident hiding behind a
/// blank.
#[test]
fn a_node_that_should_be_running_with_no_process_is_unreachable() {
    let derived = Derived::default();
    let crashed = HealthInputs {
        process_running: false,
        wants_running: true,
        uptime_seconds: None,
        latest: None,
        ..inputs(None, &derived)
    };
    let verdict = classify(&crashed);
    assert_eq!(verdict.state(), HealthState::Unreachable);
    assert!(verdict.reason().contains("no process"));

    // Still inside its grace, it is starting rather than failed.
    let coming_up = HealthInputs {
        process_running: false,
        wants_running: true,
        uptime_seconds: Some(20),
        latest: None,
        ..inputs(None, &derived)
    };
    assert_eq!(classify(&coming_up).state(), HealthState::Starting);

    // And a node the operator stopped is still not news.
    let stopped = HealthInputs {
        process_running: false,
        wants_running: false,
        latest: None,
        ..inputs(None, &derived)
    };
    assert_eq!(classify(&stopped).state(), HealthState::Stopped);
}

/// Never having looked must never read as having looked and found nothing.
#[test]
fn a_node_that_has_never_been_checked_is_unknown_not_healthy() {
    let derived = Derived::default();
    let verdict = classify(&inputs(None, &derived));
    assert_eq!(verdict.state(), HealthState::Unknown);
    assert_ne!(verdict.state().tone(), crate::observe::HealthTone::Good);
    assert!(verdict.reason().contains("not been checked"));
}

/// A node with RPC switched off is not a node in trouble, and the next step is
/// the editor rather than the log.
#[test]
fn a_node_with_rpc_disabled_is_unknown_and_points_at_the_setting() {
    let latest = NodeSample::not_observable("node-1", NOW);
    let derived = Derived::default();
    let verdict = classify(&inputs(Some(&latest), &derived));
    assert_eq!(verdict.state(), HealthState::Unknown);
    assert!(verdict.reason().contains("RPC is disabled"));
    let NextStep::Here { href, .. } = verdict.next_action() else {
        unreachable!(
            "this is fixable from the console: {:?}",
            verdict.next_action()
        )
    };
    assert!(href.contains("/edit"), "{href}");
}

/// A stale reading is not evidence of anything current.
#[test]
fn a_reading_older_than_the_policy_allows_is_unknown() {
    let latest = answering(6_245_100, NOW - 600);
    let derived = Derived::default();
    let verdict = classify(&inputs(Some(&latest), &derived));
    assert_eq!(verdict.state(), HealthState::Unknown);
    assert!(verdict.reason().contains("600s old"));
}

/// A client that takes minutes to open its store has not failed to answer; it
/// has not been asked yet.
#[test]
fn a_node_inside_its_startup_grace_is_starting_not_unreachable() {
    let mut latest = NodeSample::unreachable(
        "node-1",
        NOW,
        "http://127.0.0.1:10332",
        NotSampled::CallFailed {
            method: "getblockcount",
            detail: "connection refused".to_string(),
        },
    );
    latest.head_ok = false;
    let derived = Derived::default();

    let mut starting = inputs(Some(&latest), &derived);
    starting.uptime_seconds = Some(20);
    starting.starting_grace_seconds = 180;
    assert_eq!(classify(&starting).state(), HealthState::Starting);

    // Past the grace, with enough failures, it is genuinely unreachable.
    let mut expired = inputs(Some(&latest), &derived);
    expired.uptime_seconds = Some(600);
    expired.starting_grace_seconds = 180;
    expired.consecutive_failures = 3;
    let verdict = classify(&expired);
    assert_eq!(verdict.state(), HealthState::Unreachable);
    assert!(verdict.reason().contains("connection refused"));
}

/// A node the operator stopped is not a problem to report.
#[test]
fn a_stopped_node_is_not_news() {
    let derived = Derived::default();
    let mut stopped = inputs(None, &derived);
    stopped.process_running = false;
    stopped.wants_running = false;
    let verdict = classify(&stopped);
    assert_eq!(verdict.state(), HealthState::Stopped);
    assert!(!verdict.state().needs_attention());
}

#[test]
fn slow_rpc_and_thin_peering_are_degraded_not_healthy() {
    let mut slow = answering(6_245_100, NOW);
    slow.head_latency_ms = Some(2_400);
    let derived = Derived::default();
    let verdict = classify(&inputs(Some(&slow), &derived));
    assert_eq!(verdict.state(), HealthState::Degraded);
    assert!(verdict.reason().contains("2400ms"));

    let mut thin = answering(6_245_100, NOW);
    thin.peers_connected = Observation::Known(
        1,
        Evidence::recorded("getconnectioncount", "result", "1", "", NOW),
    );
    assert_eq!(
        classify(&inputs(Some(&thin), &derived)).state(),
        HealthState::Degraded
    );
}

/// Everything working reads as healthy, and says what it checked.
#[test]
fn a_node_at_the_head_and_answering_is_healthy() {
    let latest = answering(6_245_100, NOW);
    let derived = Derived {
        height_unchanged_seconds: Some(8),
        head_lag: Some(0),
        chain_lag_seconds: Some(9),
        ..Derived::default()
    };
    let verdict = classify(&inputs(Some(&latest), &derived));
    assert_eq!(verdict.state(), HealthState::Healthy);
    assert!(verdict.reason().contains("6245100"));
    assert!(!verdict.evidence().is_empty());
}

/// Every state the chain can produce carries somewhere to go.
#[test]
fn no_verdict_is_a_dead_end() {
    let latest = answering(6_245_100, NOW);
    let cases: Vec<Derived> = vec![
        Derived::default(),
        Derived {
            height_unchanged_seconds: Some(600),
            ..Derived::default()
        },
        Derived {
            head_lag: Some(5_000),
            ..Derived::default()
        },
    ];
    for derived in &cases {
        let verdict = classify(&inputs(Some(&latest), derived));
        match verdict.next_action() {
            NextStep::Here { label, href } => {
                assert!(!label.is_empty() && href.starts_with('/'), "{label} {href}")
            }
            NextStep::External { text } => assert!(!text.is_empty()),
        }
    }
}
