use super::*;

pub(in crate::repository::schema) fn apply_migrations(connection: &Connection) -> Result<()> {
    add_column_if_missing(connection, "rpc_health_checks", "syncing", "INTEGER")?;
    add_column_if_missing(connection, "rpc_health_checks", "observed_pid", "INTEGER")?;
    add_column_if_missing(
        connection,
        "rpc_health_checks",
        "network_observation",
        "TEXT NOT NULL DEFAULT '{}'",
    )?;
    add_column_if_missing(
        connection,
        "nodes",
        "runtime_version",
        "TEXT NOT NULL DEFAULT 'latest'",
    )?;
    add_column_if_missing(
        connection,
        "nodes",
        "storage_engine",
        "TEXT NOT NULL DEFAULT 'leveldb'",
    )?;
    add_column_if_missing(
        connection,
        "nodes",
        "rpc_port",
        "INTEGER NOT NULL DEFAULT 10332",
    )?;
    add_column_if_missing(
        connection,
        "nodes",
        "p2p_port",
        "INTEGER NOT NULL DEFAULT 10333",
    )?;
    add_column_if_missing(connection, "nodes", "ws_port", "INTEGER")?;
    add_column_if_missing(connection, "fast_sync_snapshots", "source_url", "TEXT")?;
    add_column_if_missing(
        connection,
        "fast_sync_snapshots",
        "download_file_name",
        "TEXT",
    )?;
    add_column_if_missing(
        connection,
        "fast_sync_snapshots",
        "download_max_bytes",
        "INTEGER NOT NULL DEFAULT 68719476736",
    )?;
    add_column_if_missing(
        connection,
        "runtime_installations",
        "signature_verified",
        "INTEGER NOT NULL DEFAULT 0",
    )?;
    add_column_if_missing(
        connection,
        "runtime_installations",
        "signer_public_key",
        "TEXT",
    )?;
    add_column_if_missing(
        connection,
        "operations",
        "subject_kind",
        "TEXT NOT NULL DEFAULT 'node'",
    )?;
    add_column_if_missing(
        connection,
        "operations",
        "subject_id",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    add_column_if_missing(
        connection,
        "operations",
        "operation_kind",
        "TEXT NOT NULL DEFAULT 'legacy'",
    )?;
    add_column_if_missing(
        connection,
        "operations",
        "phase",
        "TEXT NOT NULL DEFAULT 'requested'",
    )?;
    add_column_if_missing(connection, "operations", "desired_state", "TEXT")?;
    add_column_if_missing(
        connection,
        "operations",
        "generation",
        "INTEGER NOT NULL DEFAULT 0",
    )?;
    add_column_if_missing(
        connection,
        "operations",
        "fencing_token",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    add_column_if_missing(connection, "operations", "pid", "INTEGER")?;
    add_column_if_missing(connection, "operations", "process_started_at", "INTEGER")?;
    add_column_if_missing(connection, "operations", "last_error", "TEXT")?;
    connection.execute(
        "UPDATE operations SET subject_id = COALESCE(node_id, '')
         WHERE subject_id = ''",
        [],
    )?;
    Ok(())
}
