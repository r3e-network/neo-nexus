use anyhow::Result;
use rusqlite::Connection;

mod inventory;
mod observability;
mod runtime_assets;

pub(in crate::repository::schema) fn create_tables(connection: &Connection) -> Result<()> {
    inventory::create_inventory_tables(connection)?;
    observability::create_observability_tables(connection)?;
    runtime_assets::create_runtime_asset_tables(connection)?;
    // The five signer_* tables are deliberately not created here, and not dropped
    // either: the custody rows live in the signer service's database now, while a
    // workspace that predates the split still carries its own — history an
    // upgrade must not delete on the operator's behalf (§7 step 2). Nothing in
    // this process reads or writes them any more.
    Ok(())
}
