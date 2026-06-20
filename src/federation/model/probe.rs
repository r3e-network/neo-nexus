use super::RemoteProbeStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteServerProbeReport {
    pub profile_id: String,
    pub profile_name: String,
    pub base_url: String,
    pub checked_at_unix: u64,
    pub status: RemoteProbeStatus,
    pub total_nodes: Option<u64>,
    pub running_nodes: Option<u64>,
    pub syncing_nodes: Option<u64>,
    pub error_nodes: Option<u64>,
    pub total_blocks: Option<u64>,
    pub total_peers: Option<u64>,
    pub public_node_count: Option<u64>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteServerProbeRecord {
    pub id: i64,
    pub remote_server_id: String,
    pub remote_server_name: String,
    pub base_url: String,
    pub checked_at_unix: u64,
    pub status: RemoteProbeStatus,
    pub total_nodes: Option<u64>,
    pub running_nodes: Option<u64>,
    pub syncing_nodes: Option<u64>,
    pub error_nodes: Option<u64>,
    pub total_blocks: Option<u64>,
    pub total_peers: Option<u64>,
    pub public_node_count: Option<u64>,
    pub message: String,
}
