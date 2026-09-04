//! Chain-derived gauges: the newest RPC verdict of every probed node.
//!
//! Process metrics say whether a binary is alive; these say whether the chain
//! it serves is moving. A node whose process runs but whose height freezes is
//! exactly the failure process metrics cannot see.

use crate::metrics::types::MetricsSnapshot;
use crate::rpc_health::{RpcHealthRecord, RpcHealthStatus};

use super::super::text::{push_header, push_sample};

pub(in crate::metrics::prometheus) fn push_chain_health_metrics(
    output: &mut String,
    snapshot: &MetricsSnapshot,
) {
    if snapshot.chain.is_empty() {
        return;
    }

    push_header(
        output,
        "neonexus_node_rpc_health_status",
        "Latest RPC probe verdict per node: 2 healthy, 1 degraded, 0 unreachable.",
    );
    for record in &snapshot.chain {
        push_sample(
            output,
            "neonexus_node_rpc_health_status",
            &labels(record),
            status_value(record.status),
        );
    }

    push_header(
        output,
        "neonexus_node_block_height",
        "Block count reported by the latest probe. N3 count and EVM height are normalised, so one sample means the same thing on both families.",
    );
    for record in &snapshot.chain {
        if let Some(height) = record.block_count {
            push_sample(
                output,
                "neonexus_node_block_height",
                &labels(record),
                height,
            );
        }
    }

    push_header(
        output,
        "neonexus_node_rpc_checked_at_unix",
        "Unix timestamp of the latest RPC probe, so staleness is detectable.",
    );
    for record in &snapshot.chain {
        push_sample(
            output,
            "neonexus_node_rpc_checked_at_unix",
            &labels(record),
            record.checked_at_unix,
        );
    }
}

fn labels(record: &RpcHealthRecord) -> Vec<(&'static str, String)> {
    // Deliberately no `status` label: a status change would otherwise mint a
    // new series and leave the old one stale at its last value. The verdict
    // lives in the value of `neonexus_node_rpc_health_status` alone.
    vec![
        ("node_id", record.node_id.clone()),
        ("node_name", record.node_name.clone()),
        ("endpoint", record.endpoint.clone()),
    ]
}

fn status_value(status: RpcHealthStatus) -> u8 {
    match status {
        RpcHealthStatus::Healthy => 2,
        RpcHealthStatus::Degraded => 1,
        RpcHealthStatus::Unreachable => 0,
    }
}
