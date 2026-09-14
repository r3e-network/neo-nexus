use super::*;

use crate::{
    core::node_health::NodeChainView,
    observe::{Derived, Evidence, NextStep, NodeSample, Observation, ReferenceHead},
};

const NOW: u64 = 1_770_000_000;

fn answering(height: u64, peers: Option<u32>) -> NodeSample {
    let mut sample = NodeSample::not_observable("node-1", NOW);
    sample.endpoint = "http://127.0.0.1:10332".to_string();
    sample.head_ok = true;
    sample.head_latency_ms = Some(11);
    sample.block_height = Observation::Known(
        height,
        Evidence::recorded("getblockcount", "result", height.to_string(), "", NOW),
    );
    if let Some(peers) = peers {
        sample.peers_connected = Observation::Known(
            peers,
            Evidence::recorded("getconnectioncount", "result", peers.to_string(), "", NOW),
        );
    }
    sample
}

fn view(latest: Option<NodeSample>, health: Option<NodeHealth>, derived: Derived) -> NodeChainView {
    NodeChainView {
        node_id: "node-1".to_string(),
        health,
        latest,
        derived,
        reference: ReferenceHead::SelfOnly,
    }
}

fn health(state: HealthState, since: u64, evaluated: u64) -> NodeHealth {
    NodeHealth {
        node_id: "node-1".to_string(),
        state,
        since_unix: since,
        evaluated_at_unix: evaluated,
        reason: "height 8421 has not advanced in 600s".to_string(),
        scope: None,
        cause: None,
        next: NextStep::here("Read the log", "/logs?node=node-1"),
    }
}

/// "We have not looked" must never render in the colour of "we looked and it
/// was fine". A fleet that has just started has no verdicts, and an empty cell
/// there reads as nothing wrong when it means nothing known.
#[test]
fn a_node_with_no_verdict_is_grey_and_says_so() {
    let markup = not_judged_badge();
    assert!(markup.contains("health-neutral"));
    assert!(!markup.contains("health-good"));
    assert!(markup.contains("no verdict yet"));

    let empty = health_cell(&view(None, None, Derived::default()), NOW);
    assert!(empty.contains("no verdict yet"));
}

/// Every state carries the tone its severity deserves, and `Unknown` is never
/// green whatever route it arrives by.
#[test]
fn no_state_renders_in_a_colour_that_contradicts_it() {
    for state in HealthState::ALL {
        let markup = health_badge(state);
        assert!(
            markup.contains(state.label()),
            "{state:?} must name itself: {markup}"
        );
        let green = markup.contains("health-good");
        assert_eq!(
            green,
            state == HealthState::Healthy,
            "{state:?} rendered green: {markup}"
        );
    }
}

/// A stall is described by how long it has lasted, not by how recently it was
/// noticed. "Stalled, judged four seconds ago" invites an operator to read
/// freshness as reassurance about a node that has been stuck for ten minutes.
#[test]
fn a_cell_reports_how_long_the_state_has_held_not_how_fresh_the_check_is() {
    let stalled = view(
        Some(answering(8_421, Some(6))),
        Some(health(HealthState::Stalled, NOW - 720, NOW - 10)),
        Derived::default(),
    );
    let cell = health_cell(&stalled, NOW);
    assert!(cell.contains("for 12m"), "{cell}");
    // A current judgement does not clutter the row with its own age.
    assert!(!cell.contains("judged"), "{cell}");
}

/// A verdict that has stopped being refreshed has to say so. A stale judgement
/// rendered as a current one is a console reporting a node it is no longer
/// watching.
#[test]
fn a_verdict_that_has_gone_stale_carries_its_own_age() {
    let abandoned = view(
        Some(answering(8_421, Some(6))),
        Some(health(HealthState::Healthy, NOW - 3_600, NOW - 900)),
        Derived::default(),
    );
    let cell = health_cell(&abandoned, NOW);
    assert!(cell.contains("judged 15m ago"), "{cell}");
}

/// "0 blocks behind" on a single-node chain is a number that means nothing
/// while reading as reassurance, so a node with nothing to compare against
/// says exactly that.
#[test]
fn a_node_with_no_reference_head_reports_no_lag_rather_than_zero() {
    let alone = view(
        Some(answering(42, Some(0))),
        None,
        Derived {
            head_lag: None,
            ..Derived::default()
        },
    );
    let cell = chain_cell(&alone);
    assert!(cell.contains("nothing to compare against"), "{cell}");
    assert!(!cell.contains("0 behind"), "{cell}");

    let compared = NodeChainView {
        reference: ReferenceHead::Known {
            height: 100,
            source: "seed-1".to_string(),
        },
        derived: Derived {
            head_lag: Some(58),
            ..Derived::default()
        },
        ..alone
    };
    assert!(chain_cell(&compared).contains("58 behind seed-1"));
}

/// A height that was never read renders as the reason it is missing, never as
/// a number and never as a blank.
#[test]
fn an_unread_height_renders_its_reason() {
    let disabled = view(
        Some(NodeSample::not_observable("node-1", NOW)),
        None,
        Derived::default(),
    );
    let cell = chain_cell(&disabled);
    assert!(cell.contains("RPC is disabled"), "{cell}");
    assert!(!cell.contains(">0<"), "{cell}");
}

/// The panel shows the readings the verdict was drawn from, and a reading that
/// was not taken says which absence it is. A peer count of zero is `Isolated`,
/// a state that pages someone; a client that was never asked must not enter it.
#[test]
fn the_panel_never_prints_an_unmeasured_value_as_a_number() {
    let no_peer_call = view(
        Some(answering(8_421, None)),
        Some(health(HealthState::Healthy, NOW - 60, NOW)),
        Derived::default(),
    );
    let panel = verdict_panel(&no_peer_call, NOW);
    assert!(panel.contains("Peers"), "{panel}");
    assert!(
        !panel.contains(r#"<div class="mono">0</div>"#),
        "an unread peer count rendered as zero: {panel}"
    );
    // And the figures that need history say they lack it rather than showing 0.
    assert!(panel.contains("not enough history"), "{panel}");
}

/// Advice this console cannot act on stays a sentence. A link that goes
/// nowhere is how a next step becomes a dead end at three in the morning.
#[test]
fn a_step_taken_elsewhere_is_not_rendered_as_a_button() {
    let elsewhere = view(
        Some(answering(8_421, Some(6))),
        Some(NodeHealth {
            next: NextStep::external("This needs a committee vote on chain."),
            ..health(HealthState::Degraded, NOW - 60, NOW)
        }),
        Derived::default(),
    );
    let panel = verdict_panel(&elsewhere, NOW);
    assert!(panel.contains("committee vote"));
    assert!(
        !panel.contains(r#"class="btn small primary""#),
        "rendered as a button: {panel}"
    );
}

/// A clock disagreement is reported as a clock disagreement, not as a node that
/// is impossibly fresh — a negative lag presented as freshness is the shape of
/// a bug that hides a real one.
#[test]
fn a_block_dated_in_the_future_is_named_rather_than_shown_as_freshness() {
    let skewed = view(
        Some(answering(8_421, Some(6))),
        Some(health(HealthState::Healthy, NOW - 60, NOW)),
        Derived {
            clock_suspect: true,
            chain_lag_seconds: None,
            ..Derived::default()
        },
    );
    let panel = verdict_panel(&skewed, NOW);
    assert!(panel.contains("a clock is wrong"), "{panel}");
}

/// Durations round to one unit, because an operator scanning a column wants
/// "12m" and the exact figure is in the sentence beside it.
#[test]
fn durations_round_to_a_unit_a_person_would_say() {
    assert_eq!(duration_label(0), "0s");
    assert_eq!(duration_label(59), "59s");
    assert_eq!(duration_label(60), "1m");
    assert_eq!(duration_label(761), "12m");
    assert_eq!(duration_label(3_600), "1h");
    assert_eq!(duration_label(90_000), "1d");
}

/// Operator-supplied text reaches the page escaped. A node named with a script
/// tag is a stored cross-site scripting hole in every surface that renders it.
#[test]
fn text_from_a_node_is_escaped_before_it_reaches_the_page() {
    let hostile = view(
        Some(answering(8_421, Some(6))),
        Some(NodeHealth {
            reason: "<script>alert(1)</script>".to_string(),
            cause: Some("<img src=x onerror=alert(2)>".to_string()),
            ..health(HealthState::Degraded, NOW - 60, NOW)
        }),
        Derived::default(),
    );
    let panel = verdict_panel(&hostile, NOW);
    // The escaped text still contains the words; what must not survive is a
    // tag the browser would open.
    assert!(!panel.contains("<script"), "{panel}");
    assert!(!panel.contains("<img"), "{panel}");
    assert!(panel.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert!(panel.contains("&lt;img src=x onerror=alert(2)&gt;"));
}
