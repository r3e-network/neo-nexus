//! What each node says about its chain, for whatever is scraping.
//!
//! `docs/AGENT_API.md` has documented `neonexus_node_running`,
//! `neonexus_node_block_height` and `neonexus_node_rpc_latency_seconds` for as
//! long as the file has existed. None of the three appeared anywhere in `src/`:
//! the exposition carried four workspace counters, six host gauges and four
//! per-process gauges, and its labels were node/pid/status only — nothing about
//! the chain at all. So an operator who wired an alert to the documented names
//! got a rule that never fired, which is worse than one that does not exist.
//!
//! **A value that was not read is not emitted.** Prometheus has no null, and a
//! missing series is how it says "no data" — so an unread peer count produces
//! no sample rather than a zero. Emitting zero would put a node whose client
//! does not implement `getconnectioncount` permanently into whatever alert
//! watches for isolation, which is the same fabrication this workspace spent
//! Theme 1 removing, one layer further out.

use super::super::text::{push_header, push_sample};

/// One node's chain state, flattened for exposition.
///
/// Plain data, and deliberately not built here: `metrics` must not depend on
/// the observation layer, because `core` already depends on `metrics`. The
/// caller maps its `NodeChainView` into this.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChainMetricRow {
    pub node_id: String,
    pub node_name: String,
    pub client: String,
    pub network: String,
    /// The health state's stable key, for a `state`-labelled info gauge.
    /// `None` before a node has ever been judged — which is not `unknown`, and
    /// not anything else either.
    pub health_state: Option<String>,
    /// Whether the supervisor believes a process is running. Always known, so
    /// always emitted.
    pub process_running: bool,
    pub block_height: Option<u64>,
    pub header_height: Option<u64>,
    pub peers_connected: Option<u32>,
    pub rpc_latency_ms: Option<u32>,
    pub head_lag_blocks: Option<u64>,
    pub seconds_since_height_changed: Option<u64>,
    pub seconds_since_head_block: Option<i64>,
    /// The network magic the node actually joined, as a label rather than a
    /// value: it identifies a series, it is not a quantity to graph.
    pub observed_magic: Option<u64>,
    pub sampled_at_unix: Option<u64>,
}

pub(in crate::metrics::prometheus) fn push_chain_metrics(
    output: &mut String,
    rows: &[ChainMetricRow],
) {
    if rows.is_empty() {
        return;
    }

    const FAMILIES: [(&str, &str); 8] = [
        (
            "neonexus_node_running",
            "1 when NeoNexus believes a process is running for this node, 0 otherwise.",
        ),
        (
            "neonexus_node_health",
            "1 for the node's current chain health state, carried in the state label.",
        ),
        (
            "neonexus_node_block_height",
            "Blocks the node reported holding. Absent when the height was not read.",
        ),
        (
            "neonexus_node_header_height",
            "Headers the node reported holding. Absent when not read.",
        ),
        (
            "neonexus_node_peers_connected",
            "Peers the node reported. Absent when the client was not asked or does not answer.",
        ),
        (
            "neonexus_node_rpc_latency_seconds",
            "Round trip of the node's head call, in seconds. Absent when not measured.",
        ),
        (
            "neonexus_node_head_lag_blocks",
            "Blocks behind the highest node on the same chain. Absent with nothing to compare against.",
        ),
        (
            "neonexus_node_seconds_since_height_changed",
            "Seconds since the node's height last increased. Absent without enough history.",
        ),
    ];
    for (name, help) in FAMILIES {
        push_header(output, name, help);
    }

    for row in rows {
        let labels = labels_for(row);
        push_sample(
            output,
            "neonexus_node_running",
            &labels,
            u8::from(row.process_running),
        );
        if let Some(state) = &row.health_state {
            let mut with_state = labels.clone();
            with_state.push(("state", state.clone()));
            push_sample(output, "neonexus_node_health", &with_state, 1);
        }
        push_optional(
            output,
            "neonexus_node_block_height",
            &labels,
            row.block_height,
        );
        push_optional(
            output,
            "neonexus_node_header_height",
            &labels,
            row.header_height,
        );
        push_optional(
            output,
            "neonexus_node_peers_connected",
            &labels,
            row.peers_connected,
        );
        if let Some(latency_ms) = row.rpc_latency_ms {
            // Seconds, because that is the unit Prometheus expects and the
            // documented name promises. A `_seconds` metric carrying
            // milliseconds is a trap for whoever writes the first alert.
            push_sample(
                output,
                "neonexus_node_rpc_latency_seconds",
                &labels,
                f64::from(latency_ms) / 1_000.0,
            );
        }
        push_optional(
            output,
            "neonexus_node_head_lag_blocks",
            &labels,
            row.head_lag_blocks,
        );
        push_optional(
            output,
            "neonexus_node_seconds_since_height_changed",
            &labels,
            row.seconds_since_height_changed,
        );
    }
}

/// Emit only what was read.
fn push_optional<T: std::fmt::Display>(
    output: &mut String,
    name: &'static str,
    labels: &[(&'static str, String)],
    value: Option<T>,
) {
    if let Some(value) = value {
        push_sample(output, name, labels, value);
    }
}

/// The labels every chain series carries.
///
/// `chain` is the magic the node **joined**, not the network it was configured
/// with. Two nodes labelled `network="private"` can be on different chains —
/// one of them having fallen back to compiled-in MainNet defaults — and
/// summing their heights would be meaningless.
fn labels_for(row: &ChainMetricRow) -> Vec<(&'static str, String)> {
    let mut labels = vec![
        ("node_id", row.node_id.clone()),
        ("node_name", row.node_name.clone()),
        ("client", row.client.clone()),
        ("network", row.network.clone()),
    ];
    if let Some(magic) = row.observed_magic {
        labels.push(("chain", magic.to_string()));
    }
    labels
}

#[cfg(test)]
#[path = "../../../../tests/unit/metrics/prometheus/chain/tests.rs"]
mod tests;
