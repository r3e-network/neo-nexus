use anyhow::Result;
use rusqlite::Connection;

mod inventory;
mod observability;
mod runtime_assets;

pub(in crate::repository::schema) fn create_tables(connection: &Connection) -> Result<()> {
    inventory::create_inventory_tables(connection)?;
    observability::create_observability_tables(connection)?;
    runtime_assets::create_runtime_asset_tables(connection)?;
    Ok(())
}
