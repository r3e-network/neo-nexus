use super::*;

fn evidence() -> Evidence {
    Evidence::recorded(
        "getblockcount",
        "result",
        "6245100",
        "http://127.0.0.1:10332",
        1_770_000_000,
    )
}

/// Every state has a distinct label, key, and round-trips through storage.
#[test]
fn every_health_state_is_nameable_and_storable() {
    let mut keys = std::collections::BTreeSet::new();
    let mut labels = std::collections::BTreeSet::new();
    for state in HealthState::ALL {
        assert!(
            keys.insert(state.persist_key()),
            "{state:?} shares a persist key with another state"
        );
        assert!(
            labels.insert(state.label()),
            "{state:?} shares a label with another state"
        );
        assert_eq!(
            HealthState::from_persist_key(state.persist_key()),
            Some(state)
        );
    }
    assert_eq!(HealthState::from_persist_key("nonsense"), None);
}

/// The rule the whole layer exists to hold: not having looked is not a pass.
///
/// `Unknown` was previously unrepresentable — a node that had never been probed
/// rendered identically to a passing one, because the check was
/// `rpc_port == 0 || latest.is_none_or(healthy)`.
#[test]
fn not_having_looked_is_never_coloured_as_a_pass() {
    assert_eq!(HealthState::Unknown.tone(), HealthTone::Neutral);
    assert_ne!(HealthState::Unknown.tone(), HealthTone::Good);
    assert_eq!(
        HealthState::Unknown.label(),
        "Not checked",
        "the label has to say that nothing was checked, not imply a result"
    );
    assert!(
        !HealthState::Unknown.needs_attention(),
        "an unchecked fleet must not page on every node at once"
    );
}

/// Only `Healthy` is green. Anything else reading as a pass is the defect this
/// type replaced.
#[test]
fn only_healthy_is_good() {
    for state in HealthState::ALL {
        let good = state.tone() == HealthTone::Good;
        assert_eq!(
            good,
            state == HealthState::Healthy,
            "{state:?} has tone {:?}",
            state.tone()
        );
    }
}

/// The states that mean "something is wrong and nobody has been told" are
/// exactly the ones that should pull attention.
#[test]
fn the_states_that_need_an_operator_are_the_ones_that_are_wrong() {
    let attention: Vec<HealthState> = HealthState::ALL
        .into_iter()
        .filter(|state| state.needs_attention())
        .collect();
    assert_eq!(
        attention,
        vec![
            HealthState::Unreachable,
            HealthState::Isolated,
            HealthState::Stalled,
            HealthState::Degraded
        ]
    );
    // Stopped is an operator's own decision; Syncing is progress; Starting is
    // expected; Unknown is silence. None of them is news.
    for quiet in [
        HealthState::Stopped,
        HealthState::Starting,
        HealthState::Syncing,
        HealthState::Unknown,
        HealthState::Healthy,
    ] {
        assert!(!quiet.needs_attention(), "{quiet:?} should not page anyone");
    }
}

/// The guard chain's precedence is part of the definition, so it is pinned.
/// `Isolated` before `Stalled` because zero peers *causes* the stall and the
/// operator needs the cause; `Stalled` before `Syncing` because a node that is
/// behind and not moving is not syncing.
#[test]
fn the_guard_order_puts_causes_before_their_symptoms() {
    assert!(HealthState::Isolated < HealthState::Stalled);
    assert!(HealthState::Stalled < HealthState::Syncing);
    assert!(HealthState::Unreachable < HealthState::Isolated);
    assert!(HealthState::Syncing < HealthState::Degraded);
    assert!(HealthState::Degraded < HealthState::Healthy);
}

/// A verdict must be able to show its working. A state with no evidence is
/// indistinguishable from the literals this layer replaced.
#[test]
fn a_verdict_carries_its_numbers_and_one_action() {
    let verdict = Verdict::new(
        HealthState::Stalled,
        "height 6245100 has not changed in 412s, and the newest block is 480s old",
        vec![evidence()],
        NextStep::here("Open logs", "/logs?node=node-1"),
    )
    .with_scope(StallScope::Node)
    .with_cause(Some(Cause::NoPeers));

    assert_eq!(verdict.state(), HealthState::Stalled);
    assert_eq!(verdict.scope(), Some(StallScope::Node));
    assert!(!verdict.evidence().is_empty());
    assert!(
        verdict.reason().contains("412s"),
        "the reason has to contain the numbers that decided it: {}",
        verdict.reason()
    );

    let summary = verdict.summary();
    assert!(summary.contains("Stalled"));
    assert!(summary.contains("no connected peers"));
    assert!(summary.contains("Open logs"));
}

/// `NextStep` has no empty variant, so a verdict cannot reach an operator at
/// 03:00 as advice with nowhere to go. Where this console genuinely cannot
/// help, it says what can.
#[test]
fn a_verdict_that_cannot_be_fixed_here_still_says_what_can() {
    let verdict = Verdict::new(
        HealthState::Degraded,
        "this node's key is no longer designated for the Oracle role",
        vec![evidence()],
        NextStep::external(
            "Designation is a committee-witnessed transaction; ask a committee member to re-designate this key",
        ),
    );
    let NextStep::External { text } = verdict.next_action() else {
        unreachable!(
            "this is not fixable from the console: {:?}",
            verdict.next_action()
        )
    };
    assert!(text.contains("committee"));
}

/// A suspected cause is attributed from measurements already in hand, so the
/// operator is told the answer rather than sent to look for it.
#[test]
fn a_cause_states_the_measurement_rather_than_suggesting_a_search() {
    let disk = Cause::DiskNearFull {
        free_percent: 0.8,
        free_bytes: 412 * 1024 * 1024,
    };
    let described = disk.describe();
    assert!(described.contains("99.2% full"), "{described}");
    assert!(described.contains("412"), "{described}");

    let peers = Cause::PeersBelowExpected {
        connected: 1,
        expected: 3,
    };
    assert!(peers.describe().contains('1') && peers.describe().contains('3'));
}
