use super::*;

impl Repository {
    pub(in crate::repository) fn connection(&self) -> Result<Connection> {
        let connection = Connection::open(&self.db_path)
            .with_context(|| format!("failed to open database {}", self.db_path.display()))?;
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .context("failed to enable SQLite foreign-key enforcement")?;
        Ok(connection)
    }
}
