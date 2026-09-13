//! The loop that asks the fleet what it can see, and writes down the answers.
//!
//! This replaces the automatic half of `probe_rpc_health`, which sampled **one
//! node per tick**: a twenty-node fleet learned each node's state every twenty
//! seconds at best, and a hundred-node fleet every hundred seconds. Anything
//! built on top of it — a stall detector, an attention queue — would have been
//! reporting history. The manual probe behind the Node page's "check now"
//! button is untouched; it is the scheduled one that moves here.
//!
//! It runs on its own thread rather than inside the supervision tick. A pass is
//! blocking I/O bounded by `max_nodes_per_tick × probe_timeout`, and the
//! supervision tick is what notices a crashed node and restarts it. Sharing a
//! thread would put a dozen seconds of unreachable-node timeouts in front of
//! every restart, which is the one thing the loop exists to do promptly.
//!
//! While the console still reads `rpc_health_checks`, every round is written to
//! both tables. The legacy row is derived from the same sample rather than from
//! a second probe, so the two can never disagree, and the status-change journal
//! entry an operator is used to seeing keeps arriving.

mod evaluate;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use crate::{
    events::EventKind,
    health_events::{rpc_health_event_severity, rpc_health_notice, should_record_rpc_health_event},
    observe::{
        forget_missing, head_method, run_pass, NodeSample, ObservationPolicy, Scheduler,
        SAMPLES_KEPT_PER_NODE,
    },
    rpc_health::{RpcHealthReport, RpcHealthStatus, RpcMethodHealth},
    types::NodeConfig,
};

use self::evaluate::{evaluate_fleet, HealthTracker};
use super::state::EngineState;

/// How often the thread wakes to see whether anything is due.
///
/// Waking is nearly free — the scheduler answers from memory — so this is short
/// enough that a node coming due is sampled promptly, and the actual rate is
/// set by the policy rather than by this.
const WAKE: Duration = Duration::from_secs(1);

/// What the loop carries between passes.
#[derive(Default)]
pub(super) struct ObservationState {
    scheduler: Scheduler,
    health: HealthTracker,
    /// Node ids seen, so ones that disappear can be forgotten.
    known: Vec<String>,
    /// When the fleet's health was last judged. `None` until the first pass, so
    /// a freshly started workspace evaluates immediately rather than showing
    /// nothing for a monitoring interval.
    last_evaluation: Option<Instant>,
}

/// Sample the fleet until told to stop.
pub(super) fn run_observation_loop(state: EngineState, stop: Arc<AtomicBool>) {
    let mut observation = ObservationState::default();
    while !stop.load(Ordering::Relaxed) {
        observe_once(&state, &mut observation);
        thread::sleep(WAKE);
    }
}

fn observe_once(state: &EngineState, observation: &mut ObservationState) {
    let Ok(monitor) = state.repository.load_rpc_health_monitor_policy() else {
        return;
    };
    let monitor = monitor.normalized();
    let policy = ObservationPolicy {
        enabled: monitor.enabled,
        min_period: monitor.interval_duration(),
        ..ObservationPolicy::default()
    };

    let nodes = state.nodes();
    // Done before the pass and regardless of whether sampling is enabled, so a
    // workspace that churns nodes does not grow the loop's maps for the life of
    // the process just because monitoring is switched off.
    forget_missing(&mut observation.scheduler, &nodes, &mut observation.known);
    observation.health.retain(&nodes);

    let running: Vec<NodeConfig> = nodes
        .iter()
        .filter(|node| node.status.is_running())
        .cloned()
        .collect();
    let Some(now_unix) = current_unix_time() else {
        return;
    };

    let round = run_pass(&mut observation.scheduler, &running, &policy, now_unix);
    for sample in &round.samples {
        let Some(node) = running.iter().find(|node| node.id == sample.node_id) else {
            continue;
        };
        record(state, node, sample);
    }
    if !round.samples.is_empty() {
        let _ = state
            .repository
            .prune_node_samples_keep_recent_per_node(SAMPLES_KEPT_PER_NODE);
    }

    // Judged on the monitoring interval rather than on every wake, which is
    // what makes "a state must be seen twice" mean two intervals instead of two
    // seconds. Every node is judged, not only the ones sampled: nothing polls a
    // node the workspace believes is down, so a crashed node would otherwise
    // keep the verdict it held when it died.
    if !policy.enabled {
        return;
    }
    let due = observation
        .last_evaluation
        .is_none_or(|last| last.elapsed() >= monitor.interval_duration());
    if due {
        observation.last_evaluation = Some(Instant::now());
        evaluate_fleet(
            state,
            &observation.scheduler,
            &nodes,
            &mut observation.health,
            now_unix,
        );
    }
}

fn record(state: &EngineState, node: &NodeConfig, sample: &NodeSample) {
    if state.repository.record_node_sample(sample).is_err() {
        return;
    }
    // A node with no RPC port has nothing to say in the legacy table, and the
    // probe this replaces skipped those nodes entirely. Writing an
    // endpoint-less `unreachable` row for one would turn "we cannot ask" into
    // "it did not answer" on every surface that still reads it.
    if node.rpc_port == 0 {
        return;
    }

    let report = legacy_report(node, sample);
    let previous = state
        .repository
        .latest_rpc_health(&node.id)
        .ok()
        .flatten()
        .map(|record| record.status);
    if state.repository.record_rpc_health(node, &report).is_err() {
        return;
    }
    let _ = state
        .repository
        .prune_rpc_health_keep_recent_per_node(super::probes::RPC_HEALTH_RETAIN_PER_NODE);
    if should_record_rpc_health_event(previous, report.status) {
        state.journal(
            node,
            EventKind::RpcHealthChecked,
            rpc_health_event_severity(report.status),
            format!("Automatic RPC health: {}", rpc_health_notice(&report)),
        );
    }
}

/// The same round, in the shape the console still reads.
///
/// Deliberately only two statuses. The old probe called a node `Degraded` when
/// one of its two calls failed, which meant a Neo X node — with no `getversion`
/// at all — permanently read as degraded while serving every request put to it.
/// Here the node either answered its liveness call or it did not; the classes
/// that were not due this round are absent, not evidence of ill health.
fn legacy_report(node: &NodeConfig, sample: &NodeSample) -> RpcHealthReport {
    let method = head_method(node.node_type.family());
    RpcHealthReport {
        endpoint: sample.endpoint.clone(),
        status: if sample.head_ok {
            RpcHealthStatus::Healthy
        } else {
            RpcHealthStatus::Unreachable
        },
        version: sample.client_version.value().cloned(),
        block_count: sample.block_height.value().copied(),
        methods: vec![RpcMethodHealth {
            method,
            ok: sample.head_ok,
            // On failure this is the sampler's own reason — "connection
            // refused", "HTTP 502" — carried through rather than re-derived, so
            // the row says what actually happened.
            detail: sample
                .block_height
                .render(|height| format!("block height {height}")),
        }],
    }
}

fn current_unix_time() -> Option<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|elapsed| elapsed.as_secs())
}

#[cfg(test)]
#[path = "../../tests/unit/supervision/observation/tests.rs"]
mod tests;
