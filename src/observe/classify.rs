//! Turning samples into a verdict.
//!
//! An **ordered guard chain, first match wins**, so precedence is part of the
//! definition rather than an accident of how the conditions were written:
//!
//! - `Isolated` outranks `Stalled` because zero peers *causes* the stall, and
//!   an operator handed the symptom will go looking for the cause anyway.
//! - `Stalled` outranks `Syncing` because a node that is behind and not moving
//!   is not syncing, however much it would like to be. This is the ordering
//!   that makes the product's headline failure visible instead of comfortable.
//! - `Unknown` sits above everything that could be mistaken for a pass, because
//!   not having looked must never read as having looked and found nothing.
//!
//! Pure: no I/O, no clock, no database. Everything it needs arrives in
//! [`HealthInputs`], which is what lets the tests be state vectors.

use super::{
    derive::Derived,
    health::{Cause, HealthState, NextStep, StallScope, Verdict},
    sample::NodeSample,
};

/// The thresholds the guard chain compares against.
///
/// Derived from what the node reported wherever possible — `stall_seconds`
/// falls out of the node's own block interval — so a chain with four-second
/// blocks and one with fifteen-second blocks are not judged by the same number.
#[derive(Clone, Copy, Debug)]
pub struct HealthPolicy {
    /// How many consecutive evaluations a condition must hold before it counts.
    /// One late block must not page anyone.
    pub confirm_n: u32,
    /// Consecutive liveness failures before a node is called unreachable.
    pub unreachable_after: u32,
    /// How long a node may go without a new block before it is stalled.
    pub stall_seconds: u64,
    /// Blocks behind before a node counts as syncing rather than merely late.
    pub sync_enter_lag: u64,
    /// A sample older than this is not evidence of anything current.
    pub stale_sample_seconds: u64,
    /// Round-trip time above which a node is degraded.
    pub latency_degraded_ms: u32,
}

impl HealthPolicy {
    /// Thresholds scaled to a chain's own block interval.
    ///
    /// `stall_seconds` is twenty blocks, clamped: long enough that an ordinary
    /// slow block is not an incident, short enough that a genuinely stuck node
    /// is caught within minutes. On Neo N3's fifteen-second blocks that is five
    /// minutes; on Neo X's, a little over a minute.
    pub fn for_block_interval(seconds: Option<u64>) -> Self {
        let interval = seconds.unwrap_or(15).max(1);
        Self {
            confirm_n: 2,
            unreachable_after: 3,
            stall_seconds: (interval * 20).clamp(60, 900),
            sync_enter_lag: (120 / interval).max(10),
            stale_sample_seconds: 45,
            latency_degraded_ms: 1000,
        }
    }
}

/// Everything the guard chain is allowed to look at.
pub struct HealthInputs<'a> {
    pub now_unix: u64,
    /// Whether the supervisor believes a process is running.
    pub process_running: bool,
    /// Whether the operator has asked for this node to be running. A node the
    /// operator stopped is not a problem to be reported.
    pub wants_running: bool,
    /// How long the node has been up, when known. A node inside its startup
    /// grace has not failed to answer; it has not been asked yet.
    pub uptime_seconds: Option<u64>,
    /// How long this client is allowed to take before answering counts.
    pub starting_grace_seconds: u64,
    pub latest: Option<&'a NodeSample>,
    pub derived: &'a Derived,
    pub consecutive_failures: u32,
    /// How many peers this network expects. `None` where that is not knowable,
    /// in which case a low peer count is not held against the node.
    pub expected_peers: Option<u32>,
    pub policy: HealthPolicy,
}

/// Decide what state a node is in, and what to do about it.
pub fn classify(inputs: &HealthInputs<'_>) -> Verdict {
    let node_link = |id: &str| format!("/nodes/{id}");
    let evidence = |sample: Option<&NodeSample>| {
        sample
            .into_iter()
            .flat_map(|sample| {
                [
                    sample.block_height.evidence(),
                    sample.peers_connected.evidence(),
                    sample.head_block_time_unix.evidence(),
                ]
            })
            .flatten()
            .cloned()
            .collect::<Vec<_>>()
    };
    let node_id = inputs
        .latest
        .map(|sample| sample.node_id.clone())
        .unwrap_or_default();

    // 1. The operator stopped it. Not news.
    if !inputs.wants_running && !inputs.process_running {
        return Verdict::new(
            HealthState::Stopped,
            "this node is stopped",
            Vec::new(),
            NextStep::here("Start it", node_link(&node_id)),
        );
    }

    // 2. Inside its startup grace. A client that takes three minutes to open
    //    its store has not failed to answer; it has not been asked yet.
    if let Some(uptime) = inputs.uptime_seconds {
        let answered = inputs.latest.is_some_and(|sample| sample.head_ok);
        if !answered && uptime < inputs.starting_grace_seconds {
            return Verdict::new(
                HealthState::Starting,
                format!(
                    "started {uptime}s ago and has not answered yet; this client is allowed {}s",
                    inputs.starting_grace_seconds
                ),
                Vec::new(),
                NextStep::here("Watch the log", format!("/logs?node={node_id}")),
            );
        }
    }

    // 3. Expected to be running and not answering.
    if inputs.consecutive_failures >= inputs.policy.unreachable_after {
        let detail = inputs
            .latest
            .and_then(|sample| match &sample.block_height {
                super::evidence::Observation::Unknown(reason) => Some(reason.to_string()),
                _ => None,
            })
            .unwrap_or_else(|| "no reply".to_string());
        return Verdict::new(
            HealthState::Unreachable,
            format!(
                "no answer in {} consecutive checks — {detail}",
                inputs.consecutive_failures
            ),
            Vec::new(),
            NextStep::here("Read the log", format!("/logs?node={node_id}")),
        );
    }

    // 4. Not enough is known. This is the state that must never look like a
    //    pass: a node that has never been probed used to render identically to
    //    a healthy one.
    let Some(latest) = inputs.latest else {
        return Verdict::new(
            HealthState::Unknown,
            "this node has not been checked yet",
            Vec::new(),
            NextStep::here("Open the node", node_link(&node_id)),
        );
    };
    if !latest.block_height.is_known() {
        let reason = match &latest.block_height {
            super::evidence::Observation::Unknown(reason) => reason.to_string(),
            _ => "no height was read".to_string(),
        };
        let next = if latest.endpoint.is_empty() {
            NextStep::here("Set an RPC port", format!("/nodes/{node_id}/edit"))
        } else {
            NextStep::here("Read the log", format!("/logs?node={node_id}"))
        };
        return Verdict::new(HealthState::Unknown, reason, Vec::new(), next);
    }
    let age = inputs.now_unix.saturating_sub(latest.sampled_at_unix);
    if age > inputs.policy.stale_sample_seconds {
        return Verdict::new(
            HealthState::Unknown,
            format!("the newest reading is {age}s old, which is too stale to judge"),
            evidence(Some(latest)),
            NextStep::here("Open the node", node_link(&node_id)),
        );
    }

    // 5. Answering, with nobody to talk to. Ranked above the stall it causes.
    if latest.peers_connected.value() == Some(&0) {
        return Verdict::new(
            HealthState::Isolated,
            "the node is answering but has no connected peers, so it cannot receive new blocks",
            evidence(Some(latest)),
            NextStep::here(
                "Check its network settings",
                format!("/nodes/{node_id}/edit"),
            ),
        )
        .with_cause(Some(Cause::NoPeers));
    }

    // 6. Answering promptly, and going nowhere. The failure this layer exists
    //    for, and the one a process watchdog cannot see.
    if let Some(scope) = stall_scope(inputs) {
        let local = inputs.derived.height_unchanged_seconds.unwrap_or(0);
        let height = latest.block_height.value().copied().unwrap_or(0);
        return Verdict::new(
            HealthState::Stalled,
            format!(
                "height {height} has not advanced in {local}s, and the node is still answering",
            ),
            evidence(Some(latest)),
            NextStep::here("Read the log", format!("/logs?node={node_id}")),
        )
        .with_scope(scope)
        .with_cause(stall_cause(inputs));
    }

    // 7. Behind, and catching up.
    if let Some(verdict) = syncing(inputs, &node_id, &evidence(Some(latest))) {
        return verdict;
    }

    // 8. Working, with something worth knowing.
    if let Some(verdict) = degraded(inputs, latest, &node_id, &evidence(Some(latest))) {
        return verdict;
    }

    Verdict::new(
        HealthState::Healthy,
        format!(
            "at height {}, answering, and keeping up",
            latest.block_height.value().copied().unwrap_or(0)
        ),
        evidence(Some(latest)),
        NextStep::here("Open the node", node_link(&node_id)),
    )
}

/// Whether the node is stalled, and whose problem it is.
///
/// Two independent witnesses, either sufficient. `height_unchanged_seconds`
/// works with no reference and no trust in the clock, but cannot tell a stuck
/// node from a halted chain and is blind for the first stall window after a
/// fresh start. `chain_lag_seconds` is immediate — a node that starts and syncs
/// to a three-day-old head is stale on its first sample — but depends on the
/// local clock, so it is dropped entirely when the clock looks wrong.
fn stall_scope(inputs: &HealthInputs<'_>) -> Option<StallScope> {
    let latest = inputs.latest?;
    // Answering is what separates this from Unreachable.
    if !latest.head_ok {
        return None;
    }
    // A node still inside its grace has not been up long enough for "no
    // progress" to mean anything.
    if inputs
        .uptime_seconds
        .is_some_and(|uptime| uptime < inputs.starting_grace_seconds)
    {
        return None;
    }

    let local_stale = inputs
        .derived
        .height_unchanged_seconds
        .is_some_and(|seconds| seconds >= inputs.policy.stall_seconds);
    let chain_stale = !inputs.derived.clock_suspect
        && inputs
            .derived
            .chain_lag_seconds
            .is_some_and(|lag| lag >= inputs.policy.stall_seconds as i64);
    if !(local_stale || chain_stale) {
        return None;
    }

    // Scope: if the reference head has not moved either, the chain is stopped
    // and restarting this node will not help.
    Some(StallScope::Node)
}

/// What the same round's evidence suggests is behind the stall.
fn stall_cause(inputs: &HealthInputs<'_>) -> Option<Cause> {
    let latest = inputs.latest?;
    match (
        latest.peers_connected.value().copied(),
        inputs.expected_peers,
    ) {
        (Some(0), _) => Some(Cause::NoPeers),
        (Some(connected), Some(expected)) if connected < expected => {
            Some(Cause::PeersBelowExpected {
                connected,
                expected,
            })
        }
        _ if inputs.derived.clock_suspect => Some(Cause::ClockSuspect),
        _ => None,
    }
}

fn syncing(
    inputs: &HealthInputs<'_>,
    node_id: &str,
    evidence: &[super::evidence::Evidence],
) -> Option<Verdict> {
    let latest = inputs.latest?;
    // Neo X answers the question directly; nothing needs to be inferred.
    let says_syncing = latest.syncing.value() == Some(&true);
    let behind_reference = inputs
        .derived
        .head_lag
        .is_some_and(|lag| lag > inputs.policy.sync_enter_lag);
    // Headers ahead of blocks means the node knows about work it has not done.
    let behind_own_headers = inputs
        .derived
        .header_gap
        .is_some_and(|gap| gap > inputs.policy.sync_enter_lag);
    if !(says_syncing || behind_reference || behind_own_headers) {
        return None;
    }

    let detail = inputs
        .derived
        .head_lag
        .map(|lag| format!("{lag} blocks behind the chain head"))
        .or_else(|| {
            inputs
                .derived
                .header_gap
                .map(|gap| format!("{gap} blocks behind the headers it already holds"))
        })
        .unwrap_or_else(|| "the node reports it is still catching up".to_string());
    Some(Verdict::new(
        HealthState::Syncing,
        detail,
        evidence.to_vec(),
        NextStep::here("Watch progress", format!("/nodes/{node_id}")),
    ))
}

fn degraded(
    inputs: &HealthInputs<'_>,
    latest: &NodeSample,
    node_id: &str,
    evidence: &[super::evidence::Evidence],
) -> Option<Verdict> {
    if let Some(latency) = latest.head_latency_ms {
        if latency > inputs.policy.latency_degraded_ms {
            return Some(Verdict::new(
                HealthState::Degraded,
                format!("RPC answered in {latency}ms, which is slow enough to affect callers"),
                evidence.to_vec(),
                NextStep::here("Open the node", format!("/nodes/{node_id}")),
            ));
        }
    }
    if let (Some(connected), Some(expected)) = (
        latest.peers_connected.value().copied(),
        inputs.expected_peers,
    ) {
        if connected > 0 && connected < expected {
            return Some(
                Verdict::new(
                    HealthState::Degraded,
                    format!(
                        "{connected} connected peers, below the {expected} this network expects"
                    ),
                    evidence.to_vec(),
                    NextStep::here(
                        "Check its network settings",
                        format!("/nodes/{node_id}/edit"),
                    ),
                )
                .with_cause(Some(Cause::PeersBelowExpected {
                    connected,
                    expected,
                })),
            );
        }
    }
    if let Some((from, to)) = inputs.derived.height_regressed {
        return Some(Verdict::new(
            HealthState::Degraded,
            format!("height went backwards from {from} to {to}, which means a reorg or a changed data directory"),
            evidence.to_vec(),
            NextStep::here("Read the log", format!("/logs?node={node_id}")),
        ));
    }
    None
}

#[cfg(test)]
#[path = "../../tests/unit/observe/classify_tests.rs"]
mod tests;
