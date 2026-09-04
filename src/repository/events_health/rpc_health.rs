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
                version, block_count, message, syncing, network_observation, observed_pid
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                checked_at_unix,
                &node.id,
                &node.name,
                &report.endpoint,
                report.status.to_string(),
                report.version.as_deref(),
                report.block_count,
                report.message(),
                report.syncing,
                serde_json::to_string(&report.network)?,
                node.pid,
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
                        version, block_count, message, syncing, network_observation, observed_pid
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
                    version, block_count, message, syncing, network_observation, observed_pid
             FROM rpc_health_checks
             WHERE node_id = ?1
             ORDER BY checked_at_unix DESC, id DESC
             LIMIT ?2",
        )?;
        let rows = statement.query_map(params![node_id, limit], rpc_health_record_from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to load RPC health records")
    }

    /// The newest probe verdict of every probed node, one row each.
    ///
    /// This is what the Prometheus exposition needs: block height and health
    /// per node without walking the fleet node by node. A node that has never
    /// been probed has no series here — Prometheus treats an absent sample and
    /// a never-observed one the same way.
    pub fn latest_rpc_health_all_nodes(&self) -> Result<Vec<RpcHealthRecord>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT r.id, r.checked_at_unix, r.node_id, r.node_name, r.endpoint, r.status,
                    r.version, r.block_count, r.message, r.syncing, r.network_observation, r.observed_pid
             FROM rpc_health_checks r
             WHERE r.id = (
                 SELECT newest.id FROM rpc_health_checks newest
                 WHERE newest.node_id = r.node_id
                 ORDER BY newest.checked_at_unix DESC, newest.id DESC LIMIT 1
             )
             ORDER BY r.node_name",
        )?;
        let rows = statement.query_map([], rpc_health_record_from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to load latest RPC health records")
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
