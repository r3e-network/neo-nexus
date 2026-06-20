use super::*;

pub(in crate::repository::schema) fn apply_migrations(connection: &Connection) -> Result<()> {
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
    Ok(())
}
