use serde_json::Value;

use super::RemoteProbeStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedRemoteStatus {
    pub total_nodes: Option<u64>,
    pub running_nodes: Option<u64>,
    pub syncing_nodes: Option<u64>,
    pub error_nodes: Option<u64>,
    pub total_blocks: Option<u64>,
    pub total_peers: Option<u64>,
}

pub fn parse_public_status(body: &Value) -> ParsedRemoteStatus {
    let status = body.get("status").unwrap_or(body);
    ParsedRemoteStatus {
        total_nodes: json_u64(status, "totalNodes"),
        running_nodes: json_u64(status, "runningNodes"),
        syncing_nodes: json_u64(status, "syncingNodes"),
        error_nodes: json_u64(status, "errorNodes"),
        total_blocks: json_u64(status, "totalBlocks"),
        total_peers: json_u64(status, "totalPeers"),
    }
}

pub(super) fn remote_probe_status(
    error_nodes: Option<u64>,
    running_nodes: Option<u64>,
) -> RemoteProbeStatus {
    if error_nodes.unwrap_or_default() > 0 {
        RemoteProbeStatus::Degraded
    } else if running_nodes.unwrap_or_default() > 0 {
        RemoteProbeStatus::Healthy
    } else {
        RemoteProbeStatus::Degraded
    }
}

pub(super) fn format_remote_probe_message(
    name: &str,
    status: RemoteProbeStatus,
    parsed: &ParsedRemoteStatus,
    public_node_count: Option<u64>,
) -> String {
    let total = parsed.total_nodes.or(public_node_count).unwrap_or_default();
    let running = parsed.running_nodes.unwrap_or_default();
    let errors = parsed.error_nodes.unwrap_or_default();
    let blocks = parsed.total_blocks.unwrap_or_default();
    format!(
        "{name} federation probe {}: {running}/{total} running, {errors} errors, {blocks} blocks",
        status.label()
    )
}

fn json_u64(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(|value| {
        value
            .as_u64()
            .or_else(|| value.as_i64().and_then(|number| u64::try_from(number).ok()))
            .or_else(|| value.as_str()?.parse().ok())
    })
}
