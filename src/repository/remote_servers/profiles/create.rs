use super::*;

impl Repository {
    pub fn create_remote_server(
        &self,
        input: NewRemoteServerProfile,
    ) -> Result<RemoteServerProfile> {
        let input = normalized_remote_input(&input)?;
        let now = current_unix_time()?;
        let profile = RemoteServerProfile {
            id: format!("remote-{}", Uuid::new_v4()),
            name: input.name,
            base_url: input.base_url,
            description: input.description,
            enabled: input.enabled,
            created_at_unix: now,
            updated_at_unix: now,
        };
        self.upsert_remote_server_profile(&profile)?;
        Ok(profile)
    }

    pub fn upsert_remote_server_profile(&self, profile: &RemoteServerProfile) -> Result<()> {
        validate_remote_server_profile(profile)?;
        let input = normalized_remote_input(&NewRemoteServerProfile {
            name: profile.name.clone(),
            base_url: profile.base_url.clone(),
            description: profile.description.clone(),
            enabled: profile.enabled,
        })?;
        let connection = self.connection()?;
        connection.execute(
            "INSERT INTO remote_servers (
                id, name, base_url, description, enabled, created_at_unix, updated_at_unix
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                base_url = excluded.base_url,
                description = excluded.description,
                enabled = excluded.enabled,
                created_at_unix = excluded.created_at_unix,
                updated_at_unix = excluded.updated_at_unix",
            params![
                &profile.id,
                input.name,
                input.base_url,
                input.description,
                input.enabled,
                profile.created_at_unix,
                profile.updated_at_unix,
            ],
        )?;
        Ok(())
    }
}
