use super::*;

impl Repository {
    pub fn update_remote_server(
        &self,
        id: &str,
        input: NewRemoteServerProfile,
    ) -> Result<RemoteServerProfile> {
        let input = normalized_remote_input(&input)?;
        let connection = self.connection()?;
        let created_at_unix = connection
            .query_row(
                "SELECT created_at_unix FROM remote_servers WHERE id = ?1",
                params![id],
                |row| row.get::<_, u64>(0),
            )
            .with_context(|| format!("remote server {id} was not found"))?;
        let updated_at_unix = current_unix_time()?.max(created_at_unix);
        connection.execute(
            "UPDATE remote_servers
             SET name = ?1,
                 base_url = ?2,
                 description = ?3,
                 enabled = ?4,
                 updated_at_unix = ?5
             WHERE id = ?6",
            params![
                input.name,
                input.base_url,
                input.description,
                input.enabled,
                updated_at_unix,
                id,
            ],
        )?;
        self.get_remote_server(&connection, id)
    }

    pub fn set_remote_server_enabled(
        &self,
        id: &str,
        enabled: bool,
    ) -> Result<RemoteServerProfile> {
        let connection = self.connection()?;
        let updated_at_unix = current_unix_time()?;
        let changed = connection.execute(
            "UPDATE remote_servers
             SET enabled = ?1, updated_at_unix = ?2
             WHERE id = ?3",
            params![enabled, updated_at_unix, id],
        )?;
        if changed == 0 {
            anyhow::bail!("remote server {id} was not found");
        }
        self.get_remote_server(&connection, id)
    }
}
