use super::RpcHealthStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
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
}
