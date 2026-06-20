use crate::federation::{RemoteProbeStatus, RemoteServerProbeRecord, RemoteServerProfile};

use super::parse_field;

pub(in crate::repository) fn remote_server_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<RemoteServerProfile> {
    Ok(RemoteServerProfile {
        id: row.get(0)?,
        name: row.get(1)?,
        base_url: row.get(2)?,
        description: row.get(3)?,
        enabled: row.get(4)?,
        created_at_unix: row.get(5)?,
        updated_at_unix: row.get(6)?,
    })
}

pub(in crate::repository) fn remote_server_probe_record_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<RemoteServerProbeRecord> {
    let status_raw: String = row.get(5)?;
    Ok(RemoteServerProbeRecord {
        id: row.get(0)?,
        checked_at_unix: row.get(1)?,
        remote_server_id: row.get(2)?,
        remote_server_name: row.get(3)?,
        base_url: row.get(4)?,
        status: parse_field::<RemoteProbeStatus>(&status_raw)?,
        total_nodes: row.get(6)?,
        running_nodes: row.get(7)?,
        syncing_nodes: row.get(8)?,
        error_nodes: row.get(9)?,
        total_blocks: row.get(10)?,
        total_peers: row.get(11)?,
        public_node_count: row.get(12)?,
        message: row.get(13)?,
    })
}
