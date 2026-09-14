//! High-level node-health reads for frontends. Frontends (the browser console,
//! a headless CLI) read a node's health through these operations rather than
//! reaching into the repository's row API, so the persistence layer stays behind
//! the core facade and a view does not query SQLite during paint.
//!
//! The split between what is read and what is recomputed here is deliberate.
//! The **verdict** — what state a node is in — is read, never re-derived: it is
//! decided in one place by the observation loop, and a page that decided it
//! again would eventually disagree with the list row next to it. The **numbers**
//! beside it are recomputed from the same stored rounds, because they are
//! presentation rather than judgement and because re-running a pure function
//! over an indexed query is cheaper than another table.

use std::collections::BTreeMap;

use anyhow::Result;

use crate::{
    observe::{derive, reference_heads, Derived, NodeSample},
    repository::Repository,
    rpc_health::RpcHealthRecord,
    types::NodeConfig,
};

/// The health vocabulary, re-exported so frontends reach it through the facade
/// rather than through `crate::observe` — the same boundary every other shared
/// domain service is behind.
pub use crate::observe::{
    HealthState, HealthTransition, NextStep, NodeHealth, ReferenceHead, StallScope,
};

/// How many rounds a surface looks back over.
///
/// The same window the observation loop derives across, so a page's "blocks per
/// minute" and the verdict beside it were computed from the same evidence.
pub const CHAIN_VIEW_WINDOW: usize = 64;

/// How many changes of state a node's timeline shows by default.
pub const TIMELINE_LENGTH: usize = 12;

/// Seconds as a person would say them.
///
/// Rounded to one unit: an operator scanning a column wants "12m", not
/// "12m 41s", and the precise figure is in the reason sentence beside it.
/// Shared by the console and the CLI so a duration does not read two ways.
pub fn duration_label(seconds: u64) -> String {
    match seconds {
        0..=59 => format!("{seconds}s"),
        60..=3_599 => format!("{}m", seconds / 60),
        3_600..=86_399 => format!("{}h", seconds / 3_600),
        _ => format!("{}d", seconds / 86_400),
    }
}

/// Everything a surface needs to render one node's chain state.
#[derive(Clone, Debug)]
pub struct NodeChainView {
    pub node_id: String,
    /// The stored verdict. `None` before the first evaluation — which a surface
    /// must render as "not judged yet" rather than as anything reassuring.
    pub health: Option<NodeHealth>,
    pub latest: Option<NodeSample>,
    pub derived: Derived,
    pub reference: ReferenceHead,
}

impl NodeChainView {
    /// How stale the verdict is, in seconds, when there is one.
    pub fn evaluated_seconds_ago(&self, now_unix: u64) -> Option<u64> {
        self.health
            .as_ref()
            .map(|health| health.evaluated_seconds_ago(now_unix))
    }
}

/// Assemble the chain view for a whole fleet in one pass.
///
/// Fleet-wide rather than per node because a reference head is a property of
/// the *group* of nodes on a chain: resolving one node's lag means knowing
/// where its chain's head is, and that is the highest height among its peers.
pub fn fleet_chain_view(
    repository: &Repository,
    nodes: &[NodeConfig],
    now_unix: u64,
) -> Result<Vec<NodeChainView>> {
    let mut histories: BTreeMap<String, Vec<NodeSample>> = BTreeMap::new();
    for node in nodes {
        histories.insert(
            node.id.clone(),
            repository.recent_node_samples(&node.id, CHAIN_VIEW_WINDOW)?,
        );
    }
    let references = reference_heads(nodes, &histories);
    let health: BTreeMap<String, NodeHealth> = repository
        .list_node_health()?
        .into_iter()
        .map(|health| (health.node_id.clone(), health))
        .collect();

    Ok(nodes
        .iter()
        .map(|node| {
            let history = histories.remove(&node.id).unwrap_or_default();
            let reference = references
                .get(&node.id)
                .cloned()
                .unwrap_or(ReferenceHead::SelfOnly);
            NodeChainView {
                node_id: node.id.clone(),
                health: health.get(&node.id).cloned(),
                derived: derive(&history, &reference, now_unix),
                latest: history.into_iter().next(),
                reference,
            }
        })
        .collect())
}

/// One node's chain view, resolved against the fleet it belongs to.
pub fn node_chain_view(
    repository: &Repository,
    nodes: &[NodeConfig],
    node_id: &str,
    now_unix: u64,
) -> Result<Option<NodeChainView>> {
    Ok(fleet_chain_view(repository, nodes, now_unix)?
        .into_iter()
        .find(|view| view.node_id == node_id))
}

/// A node's recent changes of state, newest first.
pub fn node_health_timeline(
    repository: &Repository,
    node_id: &str,
    limit: usize,
) -> Result<Vec<HealthTransition>> {
    repository.recent_health_transitions(node_id, limit)
}

/// The most recent rounds recorded for a node, newest first.
pub fn node_sample_history(
    repository: &Repository,
    node_id: &str,
    limit: usize,
) -> Result<Vec<NodeSample>> {
    repository.recent_node_samples(node_id, limit)
}

/// The most recent RPC health probe recorded for a node, if any.
pub fn latest_node_rpc_health(
    repository: &Repository,
    node_id: &str,
) -> Result<Option<RpcHealthRecord>> {
    repository.latest_rpc_health(node_id)
}

/// The most recent `limit` RPC health probes for a node, newest first — the
/// trend a detail panel shows beneath the current status.
pub fn node_rpc_health_history(
    repository: &Repository,
    node_id: &str,
    limit: usize,
) -> Result<Vec<RpcHealthRecord>> {
    repository.list_rpc_health(node_id, limit)
}
