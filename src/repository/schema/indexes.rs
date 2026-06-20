use super::*;

pub(in crate::repository::schema) fn create_indexes(connection: &Connection) -> Result<()> {
    connection.execute(
        "CREATE INDEX IF NOT EXISTS idx_runtime_events_recent
         ON runtime_events (occurred_at_unix DESC, id DESC)",
        [],
    )?;
    connection.execute(
        "CREATE INDEX IF NOT EXISTS idx_alert_deliveries_recent
         ON alert_deliveries (attempted_at_unix DESC, id DESC)",
        [],
    )?;
    connection.execute(
        "CREATE INDEX IF NOT EXISTS idx_rpc_health_checks_node_recent
         ON rpc_health_checks (node_id, checked_at_unix DESC, id DESC)",
        [],
    )?;
    connection.execute(
        "CREATE INDEX IF NOT EXISTS idx_remote_servers_enabled
         ON remote_servers (enabled DESC, name COLLATE NOCASE ASC)",
        [],
    )?;
    connection.execute(
        "CREATE INDEX IF NOT EXISTS idx_remote_server_probe_records_recent
         ON remote_server_probe_records (remote_server_id, checked_at_unix DESC, id DESC)",
        [],
    )?;
    Ok(())
}
