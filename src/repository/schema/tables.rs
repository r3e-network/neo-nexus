use anyhow::Result;
use rusqlite::Connection;

mod inventory;
mod observability;
mod runtime_assets;

pub(in crate::repository::schema) fn create_tables(connection: &Connection) -> Result<()> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS managed_agents (id TEXT PRIMARY KEY, record TEXT NOT NULL)",
    )?;
    connection.execute_batch("CREATE TABLE IF NOT EXISTS assistant_grants (
        id TEXT PRIMARY KEY, name TEXT NOT NULL, agent_id TEXT NOT NULL REFERENCES managed_agents(id) ON DELETE CASCADE,
        node_ids TEXT NOT NULL, all_nodes INTEGER NOT NULL, can_operate INTEGER NOT NULL, enabled INTEGER NOT NULL,
        token_sha256 TEXT NOT NULL UNIQUE)")?;
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS operations (
        id TEXT PRIMARY KEY,
        kind TEXT NOT NULL,
        node_id TEXT,
        state TEXT NOT NULL,
        subject_kind TEXT NOT NULL DEFAULT 'node',
        subject_id TEXT NOT NULL,
        operation_kind TEXT NOT NULL DEFAULT 'legacy',
        phase TEXT NOT NULL DEFAULT 'requested',
        desired_state TEXT,
        generation INTEGER NOT NULL DEFAULT 0,
        fencing_token TEXT NOT NULL DEFAULT '',
        pid INTEGER,
        process_started_at INTEGER,
        last_error TEXT,
        created_at_unix INTEGER NOT NULL,
        updated_at_unix INTEGER NOT NULL)",
    )?;
    inventory::create_inventory_tables(connection)?;
    observability::create_observability_tables(connection)?;
    runtime_assets::create_runtime_asset_tables(connection)?;
    Ok(())
}
