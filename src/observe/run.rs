//! One sampling pass over the fleet.
//!
//! Kept separate from the scheduler so the decision of *what to ask* stays
//! pure and testable, and the blocking I/O lives in one place that a caller
//! can choose where to run.

use std::time::Instant;

use super::{
    client,
    sample::{sample_node, SampleRound},
    schedule::{ObservationPolicy, Scheduler},
};
use crate::types::NodeConfig;

/// Sample whatever is due, and tell the scheduler what happened.
///
/// Returns only the nodes actually contacted this pass, so a caller can persist
/// exactly what was learned rather than rewriting rows for nodes nobody asked.
///
/// **This blocks.** Every call inside carries the policy's timeout, and the
/// pass is bounded by `max_nodes_per_tick`, so the worst case is timeout ×
/// ceiling — but that is still seconds, and it must not run on a thread that
/// something latency-sensitive is waiting on.
pub fn run_pass(
    scheduler: &mut Scheduler,
    nodes: &[NodeConfig],
    policy: &ObservationPolicy,
    now_unix: u64,
) -> SampleRound {
    let due = scheduler.due(nodes, Instant::now(), policy);
    if due.is_empty() {
        return SampleRound::default();
    }

    // One agent for the pass: `ureq` pools connections on the agent, and a
    // manager polling the same nodes every fifteen seconds should not open a
    // fresh connection each time.
    let agent = client::agent(policy.probe_timeout);
    let mut samples = Vec::with_capacity(due.len());
    for work in due {
        let Some(node) = nodes.iter().find(|node| node.id == work.node_id) else {
            continue;
        };
        let sample = sample_node(&agent, node, &work.classes, now_unix);
        scheduler.record(&node.id, &work.classes, Instant::now(), sample.head_ok);
        samples.push(sample);
    }
    SampleRound { samples }
}

/// Drop anything remembered about nodes that no longer exist.
///
/// Without this a workspace that churns nodes grows the scheduler's maps for
/// the life of the process.
pub fn forget_missing(scheduler: &mut Scheduler, nodes: &[NodeConfig], known: &mut Vec<String>) {
    known.retain(|id| {
        let present = nodes.iter().any(|node| &node.id == id);
        if !present {
            scheduler.forget(id);
        }
        present
    });
    for node in nodes {
        if !known.contains(&node.id) {
            known.push(node.id.clone());
        }
    }
}
