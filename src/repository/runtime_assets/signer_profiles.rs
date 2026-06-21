use super::*;

impl Repository {
    pub fn upsert_runtime_signer_profile(&self, profile: &RuntimeSignerProfile) -> Result<()> {
        validate_runtime_signer_profile(profile)?;
        let connection = self.connection()?;
        connection.execute(
            "INSERT INTO runtime_signer_profiles (
                id, label, ed25519_public_key, enabled, created_at_unix, last_used_at_unix
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
                label = excluded.label,
                ed25519_public_key = excluded.ed25519_public_key,
                enabled = excluded.enabled,
                created_at_unix = excluded.created_at_unix,
                last_used_at_unix = excluded.last_used_at_unix",
            params![
                &profile.id,
                &profile.label,
                &profile.ed25519_public_key,
                profile.enabled,
                profile.created_at_unix,
                profile.last_used_at_unix,
            ],
        )?;
        Ok(())
    }

    pub fn list_runtime_signer_profiles(&self) -> Result<Vec<RuntimeSignerProfile>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, label, ed25519_public_key, enabled, created_at_unix, last_used_at_unix
             FROM runtime_signer_profiles
             ORDER BY label COLLATE NOCASE ASC, id ASC",
        )?;
        let rows = statement.query_map([], runtime_signer_profile_from_row)?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to load runtime signer profiles")
    }

    pub fn mark_runtime_signer_profile_used(&self, id: &str, used_at_unix: u64) -> Result<()> {
        let connection = self.connection()?;
        let changed = connection.execute(
            "UPDATE runtime_signer_profiles
             SET last_used_at_unix = ?1
             WHERE id = ?2",
            params![used_at_unix, id],
        )?;
        ensure_affected_rows(changed, "runtime signer profile", id)?;
        Ok(())
    }

    pub fn delete_runtime_signer_profile(&self, id: &str) -> Result<()> {
        let connection = self.connection()?;
        let deleted = connection.execute(
            "DELETE FROM runtime_signer_profiles WHERE id = ?1",
            params![id],
        )?;
        ensure_affected_rows(deleted, "runtime signer profile", id)?;
        Ok(())
    }
}
