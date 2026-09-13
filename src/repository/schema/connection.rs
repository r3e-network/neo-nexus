use super::*;

impl Repository {
    pub(in crate::repository) fn connection(&self) -> Result<Connection> {
        let connection = Connection::open(&self.db_path)
            .with_context(|| format!("failed to open database {}", self.db_path.display()))?;
        connection
            .execute_batch(
                "PRAGMA foreign_keys = ON;
                 PRAGMA busy_timeout = 5000;",
            )
            .context("failed to configure SQLite connection pragmas")?;
        Ok(connection)
    }
}
