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
        );
        -- What the chain said, per node, per round.
        --
        -- Every value is nullable and nullable means *not read*, never zero.
        -- The distinction is the whole point: a peer count of 0 is Isolated, a
        -- state that pages someone, while a null peer count is a client that
        -- was not asked or does not answer that question. Collapsing them is
        -- how a healthy node comes to look like an incident.
        --
        -- `observed_magic` is the chain the node *joined*, not the one it was
        -- configured with. A node set to a private network that fell back to
        -- compiled-in MainNet defaults reports MainNet's magic here, and that
        -- mismatch is the only way to see it.
        CREATE TABLE IF NOT EXISTS node_samples (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            node_id TEXT NOT NULL,
            sampled_at_unix INTEGER NOT NULL,
            endpoint TEXT NOT NULL,
            head_ok INTEGER NOT NULL,
            head_latency_ms INTEGER,
            block_height INTEGER,
            header_height INTEGER,
            syncing INTEGER,
            head_block_time_unix INTEGER,
            peers_connected INTEGER,
            observed_magic INTEGER,
            ms_per_block INTEGER,
            mempool_capacity INTEGER,
            mempool_verified INTEGER,
            mempool_unverified INTEGER,
            validators_count INTEGER,
            client_version TEXT,
            FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
        );",
    )?;
    Ok(())
}
