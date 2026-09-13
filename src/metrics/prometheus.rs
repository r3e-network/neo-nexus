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

use std::path::Path;

use anyhow::{anyhow, Result};

use super::types::MetricsSnapshot;
use crate::config::GenerationContext;

pub(super) fn snapshot_to_text(snapshot: &MetricsSnapshot) -> String {
    let mut output = String::new();
    families::push_workspace_metrics(&mut output, snapshot);
    families::push_system_metrics(&mut output, snapshot);
    families::push_node_process_metrics(&mut output, snapshot);
    families::push_missing_process_metrics(&mut output, snapshot);
    output
}

/// The node workspace directory a generation context carries.
///
/// Adapters that write files beside the node — an exporter config, a plugin
/// manifest — need somewhere to write. `NodeConfig` carries no path of its own
/// because the workspace root belongs to whoever opened it, so the directory
/// travels with the context instead. There is no defensible default: falling
/// back to the process's current directory would scatter node files wherever
/// the workbench happened to be started from.
pub(super) fn node_dir_from_context(ctx: &GenerationContext) -> Result<&Path> {
    ctx.node_dir.as_deref().ok_or_else(|| {
        anyhow!(
            "the generation context carries no node workspace directory; \
             no file was written"
        )
    })
}
