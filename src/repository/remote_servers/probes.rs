use super::*;

impl Repository {
    pub fn record_remote_server_probe(
        &self,
        report: &RemoteServerProbeReport,
    ) -> Result<RemoteServerProbeRecord> {
        let connection = self.connection()?;
        connection.execute(
            "INSERT INTO remote_server_probe_records (
                checked_at_unix, remote_server_id, remote_server_name, base_url, status,
                total_nodes, running_nodes, syncing_nodes, error_nodes, total_blocks,
                total_peers, public_node_count, message
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                report.checked_at_unix,
                &report.profile_id,
                &report.profile_name,
                &report.base_url,
                report.status.to_string(),
                report.total_nodes,
                report.running_nodes,
                report.syncing_nodes,
                report.error_nodes,
                report.total_blocks,
                report.total_peers,
                report.public_node_count,
                &report.message,
            ],
        )?;
        let id = connection.last_insert_rowid();
        self.get_remote_server_probe_record(&connection, id)
    }

    pub fn latest_remote_server_probe(
        &self,
        remote_server_id: &str,
    ) -> Result<Option<RemoteServerProbeRecord>> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT id, checked_at_unix, remote_server_id, remote_server_name, base_url,
                        status, total_nodes, running_nodes, syncing_nodes, error_nodes,
                        total_blocks, total_peers, public_node_count, message
                 FROM remote_server_probe_records
                 WHERE remote_server_id = ?1
                 ORDER BY checked_at_unix DESC, id DESC
                 LIMIT 1",
                params![remote_server_id],
                remote_server_probe_record_from_row,
            )
            .optional()
            .context("failed to load latest remote server probe")
    }

    pub fn list_remote_server_probes(
        &self,
        remote_server_id: &str,
        limit: usize,
    ) -> Result<Vec<RemoteServerProbeRecord>> {
        let connection = self.connection()?;
        let limit = limit.clamp(1, 100) as i64;
        let mut statement = connection.prepare(
            "SELECT id, checked_at_unix, remote_server_id, remote_server_name, base_url,
                    status, total_nodes, running_nodes, syncing_nodes, error_nodes,
                    total_blocks, total_peers, public_node_count, message
             FROM remote_server_probe_records
             WHERE remote_server_id = ?1
             ORDER BY checked_at_unix DESC, id DESC
             LIMIT ?2",
        )?;
        let rows = statement.query_map(
            params![remote_server_id, limit],
            remote_server_probe_record_from_row,
        )?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to load remote server probe records")
    }

    pub fn prune_remote_server_probes_keep_recent_per_profile(
        &self,
        keep_recent: usize,
    ) -> Result<usize> {
        let mut connection = self.connection()?;
        let remote_server_ids = {
            let mut statement = connection
                .prepare("SELECT DISTINCT remote_server_id FROM remote_server_probe_records")?;
            let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
                .context("failed to load remote server probe profile ids")?
        };

        let transaction = connection.transaction()?;
        let mut deleted = 0;
        for remote_server_id in remote_server_ids {
            deleted += transaction.execute(
                "DELETE FROM remote_server_probe_records
                 WHERE remote_server_id = ?1
                   AND id NOT IN (
                       SELECT id FROM remote_server_probe_records
                       WHERE remote_server_id = ?1
                       ORDER BY checked_at_unix DESC, id DESC
                       LIMIT ?2
                   )",
                params![remote_server_id, keep_recent as i64],
            )?;
        }
        transaction.commit()?;
        Ok(deleted)
    }
}
