//! Fleet data assembly shared by the pages and the JSON API: every node plus
//! its most recent RPC health verdict, read through the core facade.

use anyhow::Result;

use crate::core::node_health;
use crate::repository::Repository;
use crate::rpc_health::{RpcHealthMonitorPolicy, RpcHealthStatus};
use crate::types::NodeConfig;

pub struct FleetRow {
    pub node: NodeConfig,
    pub rpc_health: String,
}

pub struct Fleet {
    pub rows: Vec<FleetRow>,
}

impl Fleet {
    pub fn load(repository: &Repository) -> Result<Self> {
        let nodes = repository.list_nodes()?;
        let policy = repository.load_rpc_health_monitor_policy()?;
        let now = super::time::now_unix();
        let rows = nodes
            .into_iter()
            .map(|node| {
                let rpc_health = latest_health_label(repository, &node, policy, now);
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
                crate::types::NodeStatus::Crashed => counts.crashed += 1,
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
    pub crashed: usize,
}

fn latest_health_label(
    repository: &Repository,
    node: &NodeConfig,
    policy: RpcHealthMonitorPolicy,
    now: u64,
) -> String {
    match node_health::latest_node_rpc_health(repository, &node.id) {
        Ok(Some(record)) => {
            if !node.status.is_running() {
                return format!("not running; last probe {}", record.status.label());
            }
            if !policy.enabled {
                return format!("monitor disabled; last probe {}", record.status.label());
            }
            if !record.matches_process(node) {
                return "awaiting probe for current process".into();
            }
            if !record.is_fresh(now, policy.observation_max_age_seconds()) {
                return format!("stale observation; last probe {}", record.status.label());
            }
            match record.status {
                RpcHealthStatus::Healthy => {
                    format!(
                        "healthy{}{}",
                        record
                            .block_count
                            .map(|block| format!(" · block {block}"))
                            .unwrap_or_default(),
                        record
                            .network
                            .peer_count
                            .map(|peers| format!(" · {peers} peers"))
                            .unwrap_or_default()
                    )
                }
                RpcHealthStatus::Degraded => format!("degraded · {}", record.message),
                RpcHealthStatus::Unreachable => "unreachable".to_string(),
            }
        }
        Ok(None) => "no probe yet".to_string(),
        Err(_) => "probe read failed".to_string(),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/web/fleet_health.rs"]
mod tests;
