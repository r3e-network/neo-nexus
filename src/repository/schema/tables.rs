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
    inventory::create_inventory_tables(connection)?;
    observability::create_observability_tables(connection)?;
    runtime_assets::create_runtime_asset_tables(connection)?;
    Ok(())
}
