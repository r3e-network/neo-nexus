use super::*;

impl Repository {
    pub(in crate::repository) fn get_event(
        &self,
        connection: &Connection,
        id: i64,
    ) -> Result<RuntimeEvent> {
        connection
            .query_row(
                "SELECT id, occurred_at_unix, node_id, node_name, kind, severity, message
                 FROM runtime_events
                 WHERE id = ?1",
                params![id],
                event_from_row,
            )
            .with_context(|| format!("runtime event {id} was not found"))
    }

    pub(in crate::repository) fn get_alert_delivery(
        &self,
        connection: &Connection,
        id: i64,
    ) -> Result<AlertDelivery> {
        connection
            .query_row(
                "SELECT id, event_id, attempted_at_unix, route_label, target, status,
                        http_status, message
                 FROM alert_deliveries
                 WHERE id = ?1",
                params![id],
                alert_delivery_from_row,
            )
            .with_context(|| format!("alert delivery {id} was not found"))
    }

    pub(in crate::repository) fn get_rpc_health_record(
        &self,
        connection: &Connection,
        id: i64,
    ) -> Result<RpcHealthRecord> {
        connection
            .query_row(
                "SELECT id, checked_at_unix, node_id, node_name, endpoint, status,
                        version, block_count, message
                 FROM rpc_health_checks
                 WHERE id = ?1",
                params![id],
                rpc_health_record_from_row,
            )
            .with_context(|| format!("RPC health record {id} was not found"))
    }

    pub(in crate::repository) fn get_fast_sync_snapshot(
        &self,
        connection: &Connection,
        id: &str,
    ) -> Result<FastSyncSnapshot> {
        connection
            .query_row(
                "SELECT id, label, network, node_type, source_path, source_url,
                        download_file_name, download_max_bytes, expected_sha256,
                        cached_path, verified_sha256, verified_at_unix, bytes
                 FROM fast_sync_snapshots
                 WHERE id = ?1",
                params![id],
                snapshot_from_row,
            )
            .with_context(|| format!("fast sync snapshot {id} was not found"))
    }

    pub(in crate::repository) fn get_remote_server(
        &self,
        connection: &Connection,
        id: &str,
    ) -> Result<RemoteServerProfile> {
        connection
            .query_row(
                "SELECT id, name, base_url, description, enabled, created_at_unix, updated_at_unix
                 FROM remote_servers
                 WHERE id = ?1",
                params![id],
                remote_server_from_row,
            )
            .with_context(|| format!("remote server {id} was not found"))
    }

    pub(in crate::repository) fn get_remote_server_probe_record(
        &self,
        connection: &Connection,
        id: i64,
    ) -> Result<RemoteServerProbeRecord> {
        connection
            .query_row(
                "SELECT id, checked_at_unix, remote_server_id, remote_server_name, base_url,
                        status, total_nodes, running_nodes, syncing_nodes, error_nodes,
                        total_blocks, total_peers, public_node_count, message
                 FROM remote_server_probe_records
                 WHERE id = ?1",
                params![id],
                remote_server_probe_record_from_row,
            )
            .with_context(|| format!("remote server probe record {id} was not found"))
    }
}
