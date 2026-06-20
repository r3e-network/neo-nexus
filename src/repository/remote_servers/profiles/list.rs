use super::*;

impl Repository {
    pub fn list_remote_servers(&self) -> Result<Vec<RemoteServerProfile>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, name, base_url, description, enabled, created_at_unix, updated_at_unix
             FROM remote_servers
             ORDER BY enabled DESC, name COLLATE NOCASE ASC, id ASC",
        )?;
        let rows = statement.query_map([], remote_server_from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to load remote server profiles")
    }
}
