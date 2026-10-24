mod families;
mod labels;
mod text;

pub mod neo_cli_adapter;
pub mod neo_go_adapter;
pub mod neo_rs_adapter;
pub mod neox_geth_adapter;
pub mod neox_reth_adapter;

pub use neo_cli_adapter::NeoCliMetricsExporter;
pub use neo_go_adapter::NeoGoMetricsAdapter;
pub use neo_rs_adapter::NeoRsMetricsAdapter;
pub use neox_geth_adapter::NeoXGethMetricsAdapter;
pub use neox_reth_adapter::NeoXRethMetricsAdapter;

use super::types::MetricsSnapshot;

pub use families::ChainMetricRow;

pub(super) fn snapshot_to_text(snapshot: &MetricsSnapshot) -> String {
    exposition(snapshot, &[])
}

/// The host and process families, plus what each node says about its chain.
///
/// Chain rows arrive from the caller rather than from the snapshot: a
/// `MetricsSnapshot` is a reading of *this host*, and the chain is a reading of
/// somewhere else entirely. Fusing them would put an RPC round trip in the same
/// struct as a resident set size and make either one harder to reason about.
pub fn exposition(snapshot: &MetricsSnapshot, chain: &[ChainMetricRow]) -> String {
    let mut output = String::new();
    families::push_workspace_metrics(&mut output, snapshot);
    families::push_system_metrics(&mut output, snapshot);
    families::push_node_process_metrics(&mut output, snapshot);
    families::push_missing_process_metrics(&mut output, snapshot);
    families::push_chain_metrics(&mut output, chain);
    output
}

#[cfg(test)]
#[path = "../../tests/unit/metrics/documented/tests.rs"]
mod documented_tests;
