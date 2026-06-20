use anyhow::Result;
use rusqlite::Connection;

pub(super) fn create_inventory_tables(connection: &Connection) -> Result<()> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS nodes (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            node_type TEXT NOT NULL,
            network TEXT NOT NULL,
            binary_path TEXT NOT NULL,
            args TEXT NOT NULL DEFAULT '',
            runtime_version TEXT NOT NULL DEFAULT 'latest',
            storage_engine TEXT NOT NULL DEFAULT 'leveldb',
            rpc_port INTEGER NOT NULL DEFAULT 10332,
            p2p_port INTEGER NOT NULL DEFAULT 10333,
            ws_port INTEGER,
            status TEXT NOT NULL,
            pid INTEGER
        );
        CREATE TABLE IF NOT EXISTS plugin_states (
            node_id TEXT NOT NULL,
            plugin_id TEXT NOT NULL,
            enabled INTEGER NOT NULL,
            PRIMARY KEY (node_id, plugin_id),
            FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS plugin_installations (
            node_id TEXT NOT NULL,
            plugin_id TEXT NOT NULL,
            installed_path TEXT NOT NULL,
            manifest_path TEXT NOT NULL,
            source_path TEXT NOT NULL,
            sha256 TEXT NOT NULL,
            package_bytes INTEGER NOT NULL,
            installed_files INTEGER NOT NULL,
            expanded_bytes INTEGER NOT NULL,
            installed_at_unix INTEGER NOT NULL,
            PRIMARY KEY (node_id, plugin_id),
            FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS workspace_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );",
    )?;
    Ok(())
}
