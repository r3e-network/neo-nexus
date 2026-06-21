use super::*;

impl Repository {
    pub fn upsert_neo_wallet_profile(&self, profile: &NeoWalletProfile) -> Result<()> {
        validate_neo_wallet_profile(profile)?;
        let connection = self.connection()?;
        connection.execute(
            "INSERT INTO neo_wallet_profiles (
                id, label, source_path, wallet_version, primary_address,
                contract_public_keys, wallet_sha256, account_count, encrypted_account_count,
                default_account_count, watch_only_account_count, validated_at_unix,
                last_used_at_unix
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
             ON CONFLICT(id) DO UPDATE SET
                label = excluded.label,
                source_path = excluded.source_path,
                wallet_version = excluded.wallet_version,
                primary_address = excluded.primary_address,
                contract_public_keys = excluded.contract_public_keys,
                wallet_sha256 = excluded.wallet_sha256,
                account_count = excluded.account_count,
                encrypted_account_count = excluded.encrypted_account_count,
                default_account_count = excluded.default_account_count,
                watch_only_account_count = excluded.watch_only_account_count,
                validated_at_unix = excluded.validated_at_unix,
                last_used_at_unix = excluded.last_used_at_unix",
            params![
                &profile.id,
                &profile.label,
                &profile.source_path,
                &profile.wallet_version,
                &profile.primary_address,
                encode_args(&profile.contract_public_keys),
                &profile.wallet_sha256,
                profile.account_count,
                profile.encrypted_account_count,
                profile.default_account_count,
                profile.watch_only_account_count,
                profile.validated_at_unix,
                profile.last_used_at_unix,
            ],
        )?;
        Ok(())
    }

    pub fn list_neo_wallet_profiles(&self) -> Result<Vec<NeoWalletProfile>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, label, source_path, wallet_version, primary_address,
                    contract_public_keys, wallet_sha256, account_count, encrypted_account_count,
                    default_account_count, watch_only_account_count, validated_at_unix,
                    last_used_at_unix
             FROM neo_wallet_profiles
             ORDER BY label COLLATE NOCASE ASC, id ASC",
        )?;
        let rows = statement.query_map([], neo_wallet_profile_from_row)?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to load Neo wallet profiles")
    }

    pub fn mark_neo_wallet_profile_used(&self, id: &str, used_at_unix: u64) -> Result<()> {
        let connection = self.connection()?;
        let changed = connection.execute(
            "UPDATE neo_wallet_profiles
             SET last_used_at_unix = ?1
             WHERE id = ?2",
            params![used_at_unix, id],
        )?;
        ensure_affected_rows(changed, "Neo wallet profile", id)?;
        Ok(())
    }

    pub fn delete_neo_wallet_profile(&self, id: &str) -> Result<()> {
        let connection = self.connection()?;
        let deleted =
            connection.execute("DELETE FROM neo_wallet_profiles WHERE id = ?1", params![id])?;
        ensure_affected_rows(deleted, "Neo wallet profile", id)?;
        Ok(())
    }
}
