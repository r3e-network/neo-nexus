use super::{super::text::push_gauge, MetricsSnapshot};

pub(in crate::metrics::prometheus) fn push_workspace_metrics(
    output: &mut String,
    snapshot: &MetricsSnapshot,
) {
    push_gauge(
        output,
        "neonexus_workspace_status",
        "Workspace metrics status, 1 when all running nodes have live processes.",
        &[],
        if snapshot.is_success() { 1 } else { 0 },
    );
    push_gauge(
        output,
        "neonexus_metrics_captured_at_unix",
        "Unix timestamp when NeoNexus captured this metrics snapshot.",
        &[],
        snapshot.captured_at_unix,
    );
    push_gauge(
        output,
        "neonexus_workspace_node_processes",
        "Managed node processes found by NeoNexus.",
        &[],
        snapshot.node_processes.len(),
    );
    push_gauge(
        output,
        "neonexus_workspace_missing_processes",
        "Managed nodes marked running whose process no longer exists.",
        &[],
        snapshot.missing_processes.len(),
    );
}
