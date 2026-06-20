use super::*;

impl Repository {
    pub fn upsert_runtime_catalog_profile(&self, profile: &RuntimeCatalogProfile) -> Result<()> {
        validate_runtime_catalog_profile(profile)?;
        let connection = self.connection()?;
        connection.execute(
            "INSERT INTO runtime_catalog_profiles (
                id, label, source, signature_source, ed25519_public_key, max_bytes,
                enabled, last_loaded_at_unix, last_signature_verified, last_bytes
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET
                label = excluded.label,
                source = excluded.source,
                signature_source = excluded.signature_source,
                ed25519_public_key = excluded.ed25519_public_key,
                max_bytes = excluded.max_bytes,
                enabled = excluded.enabled,
                last_loaded_at_unix = excluded.last_loaded_at_unix,
                last_signature_verified = excluded.last_signature_verified,
                last_bytes = excluded.last_bytes",
            params![
                &profile.id,
                &profile.label,
                &profile.source,
                &profile.signature_source,
                &profile.ed25519_public_key,
                profile.max_bytes,
                profile.enabled,
                profile.last_loaded_at_unix,
                profile.last_signature_verified,
                profile.last_bytes,
            ],
        )?;
        Ok(())
    }

    pub fn list_runtime_catalog_profiles(&self) -> Result<Vec<RuntimeCatalogProfile>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, label, source, signature_source, ed25519_public_key, max_bytes,
                    enabled, last_loaded_at_unix, last_signature_verified, last_bytes
             FROM runtime_catalog_profiles
             ORDER BY label COLLATE NOCASE ASC, id ASC",
        )?;
        let rows = statement.query_map([], runtime_catalog_profile_from_row)?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to load runtime catalog profiles")
    }

    pub fn mark_runtime_catalog_profile_loaded(
        &self,
        id: &str,
        load: &RuntimeCatalogLoad,
    ) -> Result<()> {
        let connection = self.connection()?;
        let changed = connection.execute(
            "UPDATE runtime_catalog_profiles
             SET last_loaded_at_unix = ?1,
                 last_signature_verified = ?2,
                 last_bytes = ?3
             WHERE id = ?4",
            params![load.loaded_at_unix, load.signature_verified, load.bytes, id],
        )?;
        if changed == 0 {
            anyhow::bail!("runtime catalog profile {id} was not found");
        }
        Ok(())
    }

    pub fn delete_runtime_catalog_profile(&self, id: &str) -> Result<()> {
        let connection = self.connection()?;
        let deleted = connection.execute(
            "DELETE FROM runtime_catalog_profiles WHERE id = ?1",
            params![id],
        )?;
        if deleted == 0 {
            anyhow::bail!("runtime catalog profile {id} was not found");
        }
        Ok(())
    }
}
