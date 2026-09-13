//! Turning rounds into a verdict, once, for everyone.
//!
//! Health is computed here and read everywhere else. The alternative — each
//! page deriving it from samples when it renders — produces surfaces that
//! eventually disagree, and an operator comparing a list row against a detail
//! page has no way to tell which of them is wrong.
//!
//! Every node is evaluated, not only the ones sampled this pass. Nothing polls
//! a node the workspace believes is down, so a crashed node would otherwise
//! keep whatever verdict it held when it died — or, on a fresh workspace, never
//! get one at all.

use std::collections::BTreeMap;

use crate::{
    events::EventKind,
    observe::{
        classify, derive, HealthInputs, HealthPolicy, HealthState, HealthTransition, NodeHealth,
        NodeSample, ReferenceHead, Scheduler, Verdict, SAMPLES_KEPT_PER_NODE,
        TRANSITIONS_KEPT_PER_NODE,
    },
    types::NodeConfig,
};

use super::super::state::EngineState;

/// How long a node may take to answer after launch before silence counts
/// against it.
///
/// A client opening a large LevelDB store takes minutes, and condemning it
/// during that window would make every restart of a mainnet node look like a
/// failure. Not yet per client: the clients differ, and the right number comes
/// from the runtime catalogue rather than from here.
const STARTUP_GRACE_SECONDS: u64 = 180;

/// How many rounds the derivations look back over.
///
/// Wide enough to cover the longest stall threshold — fifteen minutes, at the
/// default fifteen-second head period, is sixty rounds — so a node stuck for
/// longer than the window still accumulates a duration rather than resetting.
const DERIVE_WINDOW: usize = 64;

/// What the loop remembers between evaluations.
///
/// Only the debounce. The committed state lives in the database, so a restart
/// resumes from the fleet's real history rather than from an empty map that
/// would re-announce every node's state as new.
#[derive(Default)]
pub(super) struct HealthTracker {
    /// A state seen but not yet held long enough to replace the stored one.
    pending: BTreeMap<String, (HealthState, u32)>,
}

impl HealthTracker {
    /// Drop anything remembered about nodes that no longer exist.
    pub(super) fn retain(&mut self, nodes: &[NodeConfig]) {
        self.pending
            .retain(|id, _| nodes.iter().any(|node| &node.id == id));
    }
}

/// Evaluate every node and write down what changed.
pub(super) fn evaluate_fleet(
    state: &EngineState,
    scheduler: &Scheduler,
    nodes: &[NodeConfig],
    tracker: &mut HealthTracker,
    now_unix: u64,
) {
    let Ok(stored) = state.repository.list_node_health() else {
        return;
    };
    let stored: BTreeMap<String, NodeHealth> = stored
        .into_iter()
        .map(|health| (health.node_id.clone(), health))
        .collect();

    let histories: BTreeMap<String, Vec<NodeSample>> = nodes
        .iter()
        .filter_map(|node| {
            state
                .repository
                .recent_node_samples(&node.id, DERIVE_WINDOW)
                .ok()
                .map(|history| (node.id.clone(), history))
        })
        .collect();
    let references = reference_heads(nodes, &histories);

    let mut changed = false;
    for node in nodes {
        let history = histories.get(&node.id).map(Vec::as_slice).unwrap_or(&[]);
        let reference = references.get(&node.id).unwrap_or(&ReferenceHead::SelfOnly);
        let derived = derive(history, reference, now_unix);
        let policy = HealthPolicy::for_block_interval(derived.block_interval_seconds);
        let uptime = state
            .supervisor()
            .started_at_unix(&node.id)
            .map(|started| now_unix.saturating_sub(started));
        let verdict = classify(&HealthInputs {
            now_unix,
            // "A process exists" rather than "a process is serving": a node
            // still starting has one, and the startup grace above it is what
            // decides whether its silence means anything yet.
            process_running: node.status.is_active(),
            // Anything the operator has not stopped, they asked for. A node in
            // `Error` crashed; they did not ask for that.
            wants_running: !matches!(node.status, crate::types::NodeStatus::Stopped),
            uptime_seconds: uptime,
            starting_grace_seconds: STARTUP_GRACE_SECONDS,
            latest: history.first(),
            derived: &derived,
            consecutive_failures: scheduler.consecutive_failures(&node.id),
            // Not yet knowable: it needs the network's own seed list, and a
            // guessed number would make a correctly-peered node look thin.
            expected_peers: None,
            policy,
        });
        changed |= commit(
            state,
            tracker,
            stored.get(&node.id),
            node,
            &verdict,
            now_unix,
        );
    }

    if changed {
        let _ = state
            .repository
            .prune_health_transitions_keep_recent_per_node(TRANSITIONS_KEPT_PER_NODE);
    }
}

/// Store the verdict, and say so if it is new.
///
/// Returns whether a transition was recorded.
fn commit(
    state: &EngineState,
    tracker: &mut HealthTracker,
    stored: Option<&NodeHealth>,
    node: &NodeConfig,
    verdict: &Verdict,
    now_unix: u64,
) -> bool {
    let unchanged = stored.is_some_and(|stored| stored.state == verdict.state());
    if unchanged {
        tracker.pending.remove(&node.id);
        // The state held, so its start time holds with it — that is the number
        // an operator reads as "stalled for twelve minutes". Only the reason
        // and the freshness are refreshed.
        let since = stored.map_or(now_unix, |stored| stored.since_unix);
        let _ = state.repository.save_node_health(&NodeHealth::from_verdict(
            &node.id, verdict, since, now_unix,
        ));
        return false;
    }

    if needs_confirmation(verdict.state()) {
        let held = match tracker.pending.get(&node.id) {
            Some((candidate, seen)) if *candidate == verdict.state() => seen + 1,
            _ => 1,
        };
        if held < CONFIRM_EVALUATIONS {
            tracker
                .pending
                .insert(node.id.clone(), (verdict.state(), held));
            return false;
        }
    }
    tracker.pending.remove(&node.id);

    let health = NodeHealth::from_verdict(&node.id, verdict, now_unix, now_unix);
    if state.repository.save_node_health(&health).is_err() {
        return false;
    }
    let transition = HealthTransition {
        node_id: node.id.clone(),
        at_unix: now_unix,
        from: stored.map(|stored| stored.state),
        to: verdict.state(),
        reason: verdict.reason().to_string(),
    };
    let _ = state.repository.record_health_transition(&transition);
    state.journal(
        node,
        EventKind::NodeHealthChanged,
        severity_for(verdict.state()),
        transition.summary(),
    );
    true
}

/// How many consecutive evaluations a new state must survive.
///
/// Evaluation runs on the monitoring interval, so this is measured in intervals
/// rather than in seconds: one round where a node answered late must not put an
/// entry in its timeline and a webhook in someone's inbox.
const CONFIRM_EVALUATIONS: u32 = 2;

/// Whether entering this state has to be seen twice.
///
/// `Stopped` and `Starting` are read straight off the process rather than
/// inferred from evidence that could be noisy. Delaying them buys nothing and
/// leaves an operator who has just pressed Stop looking at `Unreachable`.
fn needs_confirmation(state: HealthState) -> bool {
    !matches!(state, HealthState::Stopped | HealthState::Starting)
}

fn severity_for(state: HealthState) -> crate::events::EventSeverity {
    use crate::events::EventSeverity;
    match state {
        HealthState::Unreachable | HealthState::Stalled => EventSeverity::Critical,
        HealthState::Isolated | HealthState::Degraded => EventSeverity::Warning,
        HealthState::Stopped
        | HealthState::Starting
        | HealthState::Unknown
        | HealthState::Syncing
        | HealthState::Healthy => EventSeverity::Info,
    }
}

/// Where each node's chain believes its head to be.
///
/// Grouped by the magic a node **actually joined**, read from its own
/// `getversion`, not by the `Network` it was configured with. A node set to a
/// private network that fell back to compiled-in MainNet defaults is on
/// MainNet, and comparing its height against the other private nodes' would
/// report a lag of several hundred million blocks instead of the configuration
/// error that caused it.
///
/// A chain with only one node in the workspace reports
/// [`ReferenceHead::SelfOnly`]: a node compared against itself is always at the
/// head, and "0 blocks behind" on a single-node private chain is a number that
/// means nothing while looking like reassurance.
fn reference_heads(
    nodes: &[NodeConfig],
    histories: &BTreeMap<String, Vec<NodeSample>>,
) -> BTreeMap<String, ReferenceHead> {
    let mut chains: BTreeMap<String, Vec<(&NodeConfig, u64)>> = BTreeMap::new();
    for node in nodes {
        let Some(latest) = histories.get(&node.id).and_then(|history| history.first()) else {
            continue;
        };
        let (Some(key), Some(height)) = (
            latest.chain_key(node.node_type.family()),
            latest.block_height.value().copied(),
        ) else {
            continue;
        };
        chains.entry(key).or_default().push((node, height));
    }

    let mut heads = BTreeMap::new();
    for members in chains.into_values() {
        if members.len() < 2 {
            continue;
        }
        let Some((holder, height)) = members.iter().max_by_key(|(_, height)| *height) else {
            continue;
        };
        for (node, _) in &members {
            heads.insert(
                node.id.clone(),
                ReferenceHead::Known {
                    height: *height,
                    source: holder.name.clone(),
                },
            );
        }
    }
    heads
}

/// How many rounds of history one evaluation reads per node.
///
/// Exposed so the retention that writes them and the window that reads them
/// cannot drift: asking for more history than is kept would silently narrow
/// every derivation.
const _: () = assert!(DERIVE_WINDOW <= SAMPLES_KEPT_PER_NODE);

#[cfg(test)]
#[path = "../../../tests/unit/supervision/observation/evaluate_tests.rs"]
mod tests;
