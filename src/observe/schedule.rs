//! Deciding what to ask, and what not to.
//!
//! The probe this replaces sampled **one node per tick**. A twenty-node fleet
//! therefore learned each node's state every twenty seconds at best, and a
//! hundred-node fleet every hundred seconds — by which time a stall detector
//! built on it would be reporting history.
//!
//! Two properties matter more than throughput, though:
//!
//! **A node under load must not be pushed harder by its own manager.** After
//! consecutive failures a node's interval backs off geometrically and every
//! class except liveness is dropped, so a node that has been down for an hour
//! costs a handful of requests rather than hundreds — while the operator still
//! sees a recent "last checked".
//!
//! **A tick is bounded.** Sampling is blocking I/O; without a ceiling on how
//! many nodes are contacted per pass, a fleet of unreachable nodes would hold
//! the supervision loop for timeout × fleet size, stalling restarts and alert
//! routing behind it.

use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

use super::sample::SampleClass;
use crate::types::NodeConfig;

/// How the fleet is sampled.
///
/// Defaults only, for now: these are not yet operator-editable, and the values
/// are the ones §6.2 of the design settled on. When they move into `settings`
/// they will be re-read each tick like the other policies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservationPolicy {
    pub enabled: bool,
    /// How many nodes may be contacted in one pass.
    pub max_nodes_per_tick: usize,
    /// How long a single call may take.
    pub probe_timeout: Duration,
    /// The shortest period any class may be asked at.
    ///
    /// This is the operator's configured monitoring interval, and it can only
    /// ever slow sampling down. An operator who sets a sixty-second interval is
    /// asking for less traffic against their nodes, and a per-class default of
    /// fifteen seconds must not quietly overrule that; an operator who sets ten
    /// seconds is not thereby asking for `getversion` six times a minute.
    pub min_period: Duration,
}

impl Default for ObservationPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            max_nodes_per_tick: 4,
            probe_timeout: Duration::from_secs(3),
            min_period: Duration::from_secs(15),
        }
    }
}

/// The largest multiplier a failing node's interval may reach.
///
/// At the default fifteen-second head period this caps a down node at one
/// attempt every two minutes.
const MAX_BACKOFF: u32 = 8;

/// One node's worth of work for this pass.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DueWork {
    pub node_id: String,
    pub classes: Vec<SampleClass>,
}

/// Remembers what has been asked and when, so the next pass can decide.
#[derive(Default)]
pub struct Scheduler {
    last_run: BTreeMap<(String, SampleClass), Instant>,
    /// Consecutive liveness failures, which drive the backoff.
    failures: BTreeMap<String, u32>,
}

impl Scheduler {
    /// What to sample this pass.
    ///
    /// Nodes are considered in the order given and the pass stops at the
    /// policy's ceiling, so a large fleet is covered across successive ticks
    /// rather than in one long blocking pass.
    pub fn due(
        &self,
        nodes: &[NodeConfig],
        now: Instant,
        policy: &ObservationPolicy,
    ) -> Vec<DueWork> {
        if !policy.enabled {
            return Vec::new();
        }
        let mut work = Vec::new();
        for node in nodes {
            if work.len() >= policy.max_nodes_per_tick {
                break;
            }
            // A node with no RPC port is sampled once, so that its row says
            // "RPC is disabled" rather than staying blank forever, and then
            // never again — there is nothing to ask and no cost worth paying.
            if node.rpc_port == 0 {
                if !self
                    .last_run
                    .contains_key(&(node.id.clone(), SampleClass::Head))
                {
                    work.push(DueWork {
                        node_id: node.id.clone(),
                        classes: vec![SampleClass::Head],
                    });
                }
                continue;
            }

            let backoff = self.backoff_for(&node.id);
            let classes: Vec<SampleClass> = SampleClass::ALL
                .into_iter()
                .filter(|class| {
                    // While a node is failing, ask only whether it is back.
                    // Its mempool depth is not the question.
                    if backoff > 1 && !class.is_liveness() {
                        return false;
                    }
                    self.is_due(&node.id, *class, now, backoff, policy)
                })
                .collect();
            if !classes.is_empty() {
                work.push(DueWork {
                    node_id: node.id.clone(),
                    classes,
                });
            }
        }
        work
    }

    fn is_due(
        &self,
        node_id: &str,
        class: SampleClass,
        now: Instant,
        backoff: u32,
        policy: &ObservationPolicy,
    ) -> bool {
        let period = (class.default_period() * backoff).max(policy.min_period);
        self.last_run
            .get(&(node_id.to_string(), class))
            .is_none_or(|last| now.duration_since(*last) >= period)
    }

    /// The current interval multiplier for a node.
    fn backoff_for(&self, node_id: &str) -> u32 {
        match self.failures.get(node_id).copied().unwrap_or(0) {
            0 => 1,
            failures => (1u32 << failures.min(3)).min(MAX_BACKOFF),
        }
    }

    /// Record that a pass ran, and whether the node answered.
    ///
    /// A single success clears the backoff entirely rather than stepping it
    /// down: a node that has come back should be watched closely again
    /// immediately, and the cost of being wrong is one extra request.
    pub fn record(&mut self, node_id: &str, classes: &[SampleClass], now: Instant, head_ok: bool) {
        for class in classes {
            self.last_run.insert((node_id.to_string(), *class), now);
        }
        if !classes.iter().any(|class| class.is_liveness()) {
            return;
        }
        if head_ok {
            self.failures.remove(node_id);
        } else {
            *self.failures.entry(node_id.to_string()).or_insert(0) += 1;
        }
    }

    /// How many consecutive times a node has failed to answer.
    ///
    /// The health state machine needs this to tell a single missed reply from
    /// a node that is genuinely down.
    pub fn consecutive_failures(&self, node_id: &str) -> u32 {
        self.failures.get(node_id).copied().unwrap_or(0)
    }

    /// Forget a node that no longer exists, so a deleted node's history does
    /// not keep its id alive in memory for the life of the process.
    pub fn forget(&mut self, node_id: &str) {
        self.last_run.retain(|(id, _), _| id != node_id);
        self.failures.remove(node_id);
    }
}

#[cfg(test)]
#[path = "../../tests/unit/observe/schedule_tests.rs"]
mod tests;
