//! Fleet data assembly shared by the pages and the JSON API: every node plus
//! its most recent RPC health verdict, read through the core facade.

use anyhow::Result;

use crate::core::workspace_queries::WorkspaceQueries;
use crate::rpc_health::RpcHealthStatus;
use crate::types::NodeConfig;

pub struct FleetRow {
    pub node: NodeConfig,
    pub rpc_health: String,
}

/// Name an instance the way an operator does.
///
/// A node id is a uuid. Messages that identify an instance by one — "leased to
/// node-9b404bd1-17f2-442b…" — name the offender in the only spelling the
/// operator has never seen, and leave them to go looking for which node that is.
/// Falls back to the id when the fleet does not contain it, which is better than
/// claiming there is no such instance.
pub fn instance_namer(nodes: &[NodeConfig]) -> impl Fn(&str) -> String + '_ {
    move |node_id: &str| {
        nodes
            .iter()
            .find(|node| node.id == node_id)
            .map_or_else(|| node_id.to_string(), |node| node.name.clone())
    }
}

pub struct Fleet {
    pub rows: Vec<FleetRow>,
}

impl Fleet {
    pub fn load(workspace: &WorkspaceQueries) -> Result<Self> {
        let nodes = workspace.list_nodes()?;
        let rows = nodes
            .into_iter()
            .map(|node| {
                let rpc_health = if node.rpc_port == 0 {
                    "disabled".to_string()
                } else {
                    latest_health_label(workspace, &node.id)
                };
                FleetRow { node, rpc_health }
            })
            .collect();
        Ok(Self { rows })
    }

    pub fn count_by_status(&self) -> FleetCounts {
        let mut counts = FleetCounts::default();
        for row in &self.rows {
            match row.node.status {
                crate::types::NodeStatus::Running => counts.running += 1,
                crate::types::NodeStatus::Starting => counts.starting += 1,
                crate::types::NodeStatus::Error => counts.error += 1,
                crate::types::NodeStatus::Stopped => counts.stopped += 1,
            }
        }
        counts.total = self.rows.len();
        counts
    }
}

#[derive(Default)]
pub struct FleetCounts {
    pub total: usize,
    pub running: usize,
    pub starting: usize,
    pub stopped: usize,
    pub error: usize,
}

fn latest_health_label(workspace: &WorkspaceQueries, node_id: &str) -> String {
    match workspace.latest_node_rpc_health(node_id) {
        Ok(Some(record)) => match record.status {
            RpcHealthStatus::Healthy => {
                format!(
                    "healthy{}",
                    record
                        .block_count
                        .map(|block| format!(" · block {block}"))
                        .unwrap_or_default()
                )
            }
            RpcHealthStatus::Degraded => "degraded".to_string(),
            RpcHealthStatus::Unreachable => "unreachable".to_string(),
        },
        Ok(None) => "no probe yet".to_string(),
        Err(_) => "probe read failed".to_string(),
    }
}
