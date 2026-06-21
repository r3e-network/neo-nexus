use super::*;

#[test]
fn remote_probe_history_filter_sorts_newest_first() {
    let records = vec![
        record(1, 20, RemoteProbeStatus::Healthy, "ok"),
        record(2, 40, RemoteProbeStatus::Degraded, "slow"),
        record(3, 40, RemoteProbeStatus::Healthy, "latest"),
    ];

    let rows = filter_remote_probe_history(&records, &RemoteProbeHistoryFilter::default());

    assert_eq!(rows[0].id, 3);
    assert_eq!(rows[1].id, 2);
    assert_eq!(rows[2].id, 1);
}

#[test]
fn remote_probe_history_filter_applies_status_and_query() {
    let records = vec![
        record(1, 20, RemoteProbeStatus::Healthy, "remote ok"),
        record(2, 30, RemoteProbeStatus::Unreachable, "timeout from lab"),
    ];

    let rows = filter_remote_probe_history(
        &records,
        &RemoteProbeHistoryFilter::new(Some(RemoteProbeStatus::Unreachable), "lab"),
    );

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, 2);
}

fn record(
    id: i64,
    checked_at_unix: u64,
    status: RemoteProbeStatus,
    message: &str,
) -> RemoteServerProbeRecord {
    RemoteServerProbeRecord {
        id,
        remote_server_id: "remote-a".to_string(),
        remote_server_name: "Remote A".to_string(),
        base_url: "https://remote.example".to_string(),
        checked_at_unix,
        status,
        total_nodes: Some(3),
        running_nodes: Some(2),
        syncing_nodes: Some(1),
        error_nodes: Some(0),
        total_blocks: Some(100),
        total_peers: Some(12),
        public_node_count: Some(3),
        message: message.to_string(),
    }
}
