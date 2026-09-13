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
        );
        -- The current verdict on each node, computed in one place.
        --
        -- Health is evaluated by the observation loop and read by every
        -- surface, rather than each page re-deriving it from samples. Two
        -- pages that derive the same thing separately eventually disagree, and
        -- an operator comparing a list row against a detail page has no way to
        -- tell which one is wrong.
        --
        -- `since_unix` is when this state was *entered*, not when it was last
        -- confirmed, so a surface can say `stalled for 12m` rather than
        -- `stalled, checked 15s ago` — the first is the number that decides
        -- whether to act.
        --
        -- `next_href` NULL means the step is not something this console can do;
        -- `next_label` then carries the whole sentence. Between them they make
        -- a verdict with no way forward unrepresentable.
        CREATE TABLE IF NOT EXISTS node_health_state (
            node_id TEXT PRIMARY KEY,
            state TEXT NOT NULL,
            since_unix INTEGER NOT NULL,
            evaluated_at_unix INTEGER NOT NULL,
            reason TEXT NOT NULL,
            scope TEXT,
            cause TEXT,
            next_label TEXT NOT NULL,
            next_href TEXT,
            FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
        );
        -- Every change of state, so `it was fine an hour ago` is a question the
        -- workspace can answer rather than one an operator has to reconstruct
        -- from log files.
        CREATE TABLE IF NOT EXISTS node_health_transitions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            node_id TEXT NOT NULL,
            at_unix INTEGER NOT NULL,
            from_state TEXT,
            to_state TEXT NOT NULL,
            reason TEXT NOT NULL,
            FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
        );",
    )?;
    Ok(())
}
