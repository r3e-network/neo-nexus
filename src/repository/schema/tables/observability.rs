use anyhow::Result;
use rusqlite::Connection;

pub(super) fn create_observability_tables(connection: &Connection) -> Result<()> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS remote_servers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            base_url TEXT NOT NULL UNIQUE,
            description TEXT NOT NULL DEFAULT '',
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at_unix INTEGER NOT NULL,
            updated_at_unix INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS remote_server_probe_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            checked_at_unix INTEGER NOT NULL,
            remote_server_id TEXT NOT NULL,
            remote_server_name TEXT NOT NULL,
            base_url TEXT NOT NULL,
            status TEXT NOT NULL,
            total_nodes INTEGER,
            running_nodes INTEGER,
            syncing_nodes INTEGER,
            error_nodes INTEGER,
            total_blocks INTEGER,
            total_peers INTEGER,
            public_node_count INTEGER,
            message TEXT NOT NULL,
            FOREIGN KEY (remote_server_id) REFERENCES remote_servers(id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS runtime_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            occurred_at_unix INTEGER NOT NULL,
            node_id TEXT,
            node_name TEXT,
            kind TEXT NOT NULL,
            severity TEXT NOT NULL,
            message TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS alert_deliveries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            event_id INTEGER NOT NULL,
            attempted_at_unix INTEGER NOT NULL,
            route_label TEXT NOT NULL,
            target TEXT NOT NULL,
            status TEXT NOT NULL,
            http_status INTEGER,
            message TEXT NOT NULL,
            FOREIGN KEY (event_id) REFERENCES runtime_events(id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS rpc_health_checks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            checked_at_unix INTEGER NOT NULL,
            node_id TEXT NOT NULL,
            node_name TEXT NOT NULL,
            endpoint TEXT NOT NULL,
            status TEXT NOT NULL,
            version TEXT,
            block_count INTEGER,
            message TEXT NOT NULL
        );",
    )?;
    Ok(())
}
