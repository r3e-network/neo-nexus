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
        tables::create_tables(&connection)?;
        api_tokens::create_api_token_table(&connection)?;
        indexes::create_indexes(&connection)?;
        migrations::apply_migrations(&connection)?;
        Ok(())
    }
}
