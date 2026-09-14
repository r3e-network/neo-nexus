use super::*;

mod api_tokens;
mod connection;
mod getters;
mod indexes;
mod migrations;
mod tables;

impl Repository {
    pub(in crate::repository) fn initialize(&self) -> Result<()> {
        let connection = self.connection()?;
        connection
            .execute_batch("PRAGMA journal_mode = WAL;")
            .context("failed to enable SQLite WAL journal mode")?;
        create_schema(&connection)
    }
}

/// Build the whole schema on a connection.
///
/// Taking a connection rather than a path is what lets the integrity checker
/// construct a **reference workspace in memory** and compare a real one against
/// it, instead of keeping a second hand-maintained declaration of the same
/// tables — which had already drifted, silently omitting the table whose unique
/// index prevents two nodes sharing a signing key.
pub(crate) fn create_schema(connection: &Connection) -> Result<()> {
    tables::create_tables(connection)?;
    api_tokens::create_api_token_table(connection)?;
    indexes::create_indexes(connection)?;
    migrations::apply_migrations(connection)?;
    Ok(())
}
