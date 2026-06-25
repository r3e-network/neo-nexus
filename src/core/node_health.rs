//! High-level node-health reads for frontends. Frontends (the GUI shell, a
//! headless CLI) read a node's RPC health through these operations rather than
//! reaching into the repository's row API, so the persistence layer stays behind
//! the core facade and a view does not query SQLite during paint.

use anyhow::Result;

use crate::{repository::Repository, rpc_health::RpcHealthRecord};

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
