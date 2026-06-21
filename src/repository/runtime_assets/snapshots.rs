use super::*;

impl Repository {
    pub fn upsert_fast_sync_snapshot(
        &self,
        input: NewFastSyncSnapshot,
    ) -> Result<FastSyncSnapshot> {
        validate_snapshot_input(&input)?;
        let id = input.id.trim().to_string();
        let label = input.label.trim().to_string();
        let expected_sha256 = normalize_sha256(&input.expected_sha256)?;
        let connection = self.connection()?;
        connection.execute(
            "INSERT INTO fast_sync_snapshots (
                id, label, network, node_type, source_path, source_url,
                download_file_name, download_max_bytes, expected_sha256,
                cached_path, verified_sha256, verified_at_unix, bytes
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, NULL, NULL, NULL, NULL)
             ON CONFLICT(id) DO UPDATE SET
                label = excluded.label,
                network = excluded.network,
                node_type = excluded.node_type,
                source_path = excluded.source_path,
                source_url = excluded.source_url,
                download_file_name = excluded.download_file_name,
                download_max_bytes = excluded.download_max_bytes,
                expected_sha256 = excluded.expected_sha256,
                cached_path = NULL,
                verified_sha256 = NULL,
                verified_at_unix = NULL,
                bytes = NULL",
            params![
                id,
                label,
                input.network.to_string(),
                input.node_type.to_string(),
                input.source_path.to_string_lossy(),
                input.source_url.as_deref().map(str::trim),
                input.download_file_name.as_deref().map(str::trim),
                input.download_max_bytes,
                expected_sha256,
            ],
        )?;

        self.get_fast_sync_snapshot(&connection, input.id.trim())
    }

    pub fn list_fast_sync_snapshots(&self) -> Result<Vec<FastSyncSnapshot>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, label, network, node_type, source_path, source_url,
                    download_file_name, download_max_bytes, expected_sha256,
                    cached_path, verified_sha256, verified_at_unix, bytes
             FROM fast_sync_snapshots
             ORDER BY label COLLATE NOCASE ASC, id ASC",
        )?;
        let rows = statement.query_map([], snapshot_from_row)?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to load fast sync snapshots")
    }

    pub fn mark_fast_sync_snapshot_verified(
        &self,
        id: &str,
        verification: &SnapshotVerification,
    ) -> Result<()> {
        let connection = self.connection()?;
        let changed = connection.execute(
            "UPDATE fast_sync_snapshots
             SET verified_sha256 = ?1,
                 verified_at_unix = ?2,
                 bytes = ?3
             WHERE id = ?4",
            params![
                verification.sha256,
                verification.verified_at_unix,
                verification.bytes,
                id,
            ],
        )?;
        ensure_affected_rows(changed, "fast sync snapshot", id)?;
        Ok(())
    }

    pub fn mark_fast_sync_snapshot_cached(&self, id: &str, cache: &SnapshotCache) -> Result<()> {
        let connection = self.connection()?;
        let changed = connection.execute(
            "UPDATE fast_sync_snapshots
             SET cached_path = ?1,
                 verified_sha256 = ?2,
                 verified_at_unix = ?3,
                 bytes = ?4
             WHERE id = ?5",
            params![
                cache.path.to_string_lossy(),
                cache.sha256,
                cache.cached_at_unix,
                cache.bytes,
                id,
            ],
        )?;
        ensure_affected_rows(changed, "fast sync snapshot", id)?;
        Ok(())
    }
}
