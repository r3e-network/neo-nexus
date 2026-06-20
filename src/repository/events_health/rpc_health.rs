use super::*;

impl Repository {
    pub fn record_rpc_health(
        &self,
        node: &NodeConfig,
        report: &RpcHealthReport,
    ) -> Result<RpcHealthRecord> {
        self.record_rpc_health_at(node, report, current_unix_time()?)
    }

    pub fn record_rpc_health_at(
        &self,
        node: &NodeConfig,
        report: &RpcHealthReport,
        checked_at_unix: u64,
    ) -> Result<RpcHealthRecord> {
        let connection = self.connection()?;
        connection.execute(
            "INSERT INTO rpc_health_checks (
                checked_at_unix, node_id, node_name, endpoint, status,
                version, block_count, message
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                checked_at_unix,
                &node.id,
                &node.name,
                &report.endpoint,
                report.status.to_string(),
                report.version.as_deref(),
                report.block_count,
                report.message(),
            ],
        )?;
        let id = connection.last_insert_rowid();
        self.get_rpc_health_record(&connection, id)
    }

    pub fn latest_rpc_health(&self, node_id: &str) -> Result<Option<RpcHealthRecord>> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT id, checked_at_unix, node_id, node_name, endpoint, status,
                        version, block_count, message
                 FROM rpc_health_checks
                 WHERE node_id = ?1
                 ORDER BY checked_at_unix DESC, id DESC
                 LIMIT 1",
                params![node_id],
                rpc_health_record_from_row,
            )
            .optional()
            .context("failed to load latest RPC health record")
    }

    pub fn list_rpc_health(&self, node_id: &str, limit: usize) -> Result<Vec<RpcHealthRecord>> {
        let connection = self.connection()?;
        let limit = limit.clamp(1, 100) as i64;
        let mut statement = connection.prepare(
            "SELECT id, checked_at_unix, node_id, node_name, endpoint, status,
                    version, block_count, message
             FROM rpc_health_checks
             WHERE node_id = ?1
             ORDER BY checked_at_unix DESC, id DESC
             LIMIT ?2",
        )?;
        let rows = statement.query_map(params![node_id, limit], rpc_health_record_from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to load RPC health records")
    }

    pub fn prune_rpc_health_keep_recent_per_node(&self, keep_recent: usize) -> Result<usize> {
        let mut connection = self.connection()?;
        let node_ids = {
            let mut statement =
                connection.prepare("SELECT DISTINCT node_id FROM rpc_health_checks")?;
            let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
                .context("failed to load RPC health node ids")?
        };

        let transaction = connection.transaction()?;
        let mut deleted = 0;
        for node_id in node_ids {
            deleted += transaction.execute(
                "DELETE FROM rpc_health_checks
                 WHERE node_id = ?1
                   AND id NOT IN (
                       SELECT id FROM rpc_health_checks
                       WHERE node_id = ?1
                       ORDER BY checked_at_unix DESC, id DESC
                       LIMIT ?2
                   )",
                params![node_id, keep_recent as i64],
            )?;
        }
        transaction.commit()?;
        Ok(deleted)
    }
}
