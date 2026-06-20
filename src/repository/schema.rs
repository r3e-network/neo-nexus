use super::*;

mod connection;
mod getters;
mod indexes;
mod migrations;
mod tables;

impl Repository {
    pub(in crate::repository) fn initialize(&self) -> Result<()> {
        let connection = self.connection()?;
        tables::create_tables(&connection)?;
        indexes::create_indexes(&connection)?;
        migrations::apply_migrations(&connection)?;
        Ok(())
    }
}
