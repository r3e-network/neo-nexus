use serde::Serialize;

use super::RpcHealthStatus;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RpcHealthRecord {
    pub id: i64,
    pub checked_at_unix: u64,
    pub node_id: String,
    pub node_name: String,
    pub endpoint: String,
    pub status: RpcHealthStatus,
    pub version: Option<String>,
    pub block_count: Option<u64>,
    pub message: String,
    pub syncing: Option<bool>,
    pub network: super::RpcNetworkObservation,
    pub observed_pid: Option<u32>,
}

impl RpcHealthRecord {
    pub fn is_fresh(&self, now_unix: u64, max_age_seconds: u64) -> bool {
        self.checked_at_unix <= now_unix
            && now_unix.saturating_sub(self.checked_at_unix) <= max_age_seconds
    }

    pub fn matches_process(&self, node: &crate::types::NodeConfig) -> bool {
        self.node_id == node.id && node.pid.is_some() && self.observed_pid == node.pid
    }
}
